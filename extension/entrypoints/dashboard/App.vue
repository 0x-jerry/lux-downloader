<script setup lang="ts">
import { onMounted, onUnmounted, reactive, ref } from 'vue'
import type { LuxConfig } from '../../src/shared'

type Task = {
  id: string
  state: string
  progress: {
    downloaded_bytes: number
    total_bytes: number | null
    download_rate_bps: number
    upload_rate_bps: number
  }
  spec: {
    source: { value: string }
    destination_path: string
  }
}

const tasks = ref<Task[]>([])
const taskStatus = ref('')
const createTaskStatus = ref('')
const creatingTask = ref(false)
const configStatus = ref('')
const savingConfig = ref(false)
const removeDialogOpen = ref(false)
const removeDialogTaskId = ref<string | null>(null)
const removeDeleteFile = ref(false)
const newTaskUrl = ref('')
const newTaskReferer = ref('')
let timer: number | undefined

const form = reactive<LuxConfig>({
  baseUrl: 'http://127.0.0.1:8080',
  authToken: 'change-me',
  interceptEnabled: true,
  includeCookies: true,
  includeReferer: true,
})

onMounted(async () => {
  await loadConfig()
  await loadTasks()
  timer = window.setInterval(loadTasks, 3000)
})

onUnmounted(() => {
  if (timer) {
    window.clearInterval(timer)
  }
})

async function loadTasks() {
  const response = await browser.runtime.sendMessage({ action: 'list_tasks' })
  if (!response?.ok) {
    taskStatus.value = response?.error ?? 'Failed to load tasks'
    return
  }

  tasks.value = (response.data?.items ?? []) as Task[]
  taskStatus.value = `Loaded ${tasks.value.length} tasks`
}

async function loadConfig() {
  const response = await browser.runtime.sendMessage({ action: 'get_config' })
  if (!response?.ok) {
    configStatus.value = response?.error ?? 'Failed to load config'
    return
  }

  Object.assign(form, response.data)
}

async function createTask() {
  if (!newTaskUrl.value.trim()) {
    createTaskStatus.value = 'Enter a URL first.'
    return
  }

  creatingTask.value = true
  createTaskStatus.value = 'Creating task...'

  try {
    const response = await browser.runtime.sendMessage({
      action: 'manual_add_task',
      payload: {
        url: newTaskUrl.value.trim(),
        referer: newTaskReferer.value.trim(),
      },
    })

    if (!response?.ok) {
      createTaskStatus.value = response?.error ?? 'Failed to create task'
      return
    }

    createTaskStatus.value = `Task created: ${response.data.id}`
    newTaskUrl.value = ''
    newTaskReferer.value = ''
    await loadTasks()
  } finally {
    creatingTask.value = false
  }
}

async function action(id: string, command: 'pause' | 'resume' | 'remove') {
  if (command === 'remove') {
    openRemoveDialog(id)
    return
  }

  await runTaskAction(id, command)
}

async function runTaskAction(
  id: string,
  command: 'pause' | 'resume' | 'remove',
  deleteFile?: boolean,
) {
  const response = await browser.runtime.sendMessage({
    action: 'task_action',
    payload: { id, action: command, deleteFile },
  })

  if (!response?.ok) {
    taskStatus.value = response?.error ?? `Failed to ${command}`
    return
  }

  taskStatus.value = `${command} succeeded`
  await loadTasks()
}

function openRemoveDialog(id: string) {
  removeDialogTaskId.value = id
  removeDeleteFile.value = false
  removeDialogOpen.value = true
}

function cancelRemoveDialog() {
  removeDialogOpen.value = false
  removeDialogTaskId.value = null
  removeDeleteFile.value = false
}

async function confirmRemoveDialog() {
  const id = removeDialogTaskId.value
  if (!id) {
    return
  }

  const deleteFile = removeDeleteFile.value
  cancelRemoveDialog()
  await runTaskAction(id, 'remove', deleteFile)
}

async function saveConfig() {
  savingConfig.value = true
  configStatus.value = 'Saving...'

  try {
    const saved = await browser.runtime.sendMessage({
      action: 'save_config',
      payload: form,
    })

    if (!saved?.ok) {
      configStatus.value = saved?.error ?? 'Failed to save config'
      return
    }

    const validation = await browser.runtime.sendMessage({
      action: 'validate_config',
      payload: form,
    })

    if (!validation?.ok) {
      configStatus.value = `Saved, but validation failed: ${validation.error}`
      return
    }

    configStatus.value = 'Saved and validated successfully.'
  } finally {
    savingConfig.value = false
  }
}

function progressText(task: Task): string {
  const downloaded = formatBytes(task.progress.downloaded_bytes)
  const total = task.progress.total_bytes ? formatBytes(task.progress.total_bytes) : '?'
  const downRate = formatBytes(task.progress.download_rate_bps) + '/s'
  const upRate = formatBytes(task.progress.upload_rate_bps) + '/s'
  return `${downloaded} / ${total} (Down ${downRate}, Up ${upRate})`
}

function canPause(task: Task): boolean {
  const state = task.state.toLowerCase()
  return ['queued', 'metadata_fetching', 'downloading', 'seeding'].includes(state)
}

function canResume(task: Task): boolean {
  return task.state.toLowerCase() === 'paused'
}

function taskTitle(taskId: string | null): string {
  if (!taskId) {
    return ''
  }

  const task = tasks.value.find((item) => item.id === taskId)
  return task?.spec.destination_path || taskId
}

function formatBytes(bytes: number): string {
  if (!bytes) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let index = 0

  while (size >= 1024 && index < units.length - 1) {
    size /= 1024
    index += 1
  }

  return `${size.toFixed(size >= 10 ? 0 : 1)} ${units[index]}`
}
</script>

<template>
  <main class="dashboard">
    <header>
      <h1>Lux Dashboard</h1>
      <button @click="loadTasks">Refresh Tasks</button>
    </header>

    <div class="layout">
      <section class="card settings">
        <h2>Settings</h2>

        <label>
          Lux Base URL
          <input v-model="form.baseUrl" type="url" placeholder="http://127.0.0.1:8080" />
        </label>

        <label>
          Bearer Token
          <input v-model="form.authToken" type="text" />
        </label>

        <label class="checkbox">
          <input v-model="form.interceptEnabled" type="checkbox" />
          Enable automatic link interception
        </label>

        <label class="checkbox">
          <input v-model="form.includeReferer" type="checkbox" />
          Include referer header when intercepting
        </label>

        <label class="checkbox">
          <input v-model="form.includeCookies" type="checkbox" />
          Include cookies for intercepted link domain
        </label>

        <div class="row">
          <button :disabled="savingConfig" @click="saveConfig">Save & Validate</button>
          <p class="status">{{ configStatus }}</p>
        </div>
      </section>

      <section class="card tasks">
        <h2>Tasks</h2>
        <form class="add-task-form" @submit.prevent="createTask">
          <label>
            URL
            <input
              v-model="newTaskUrl"
              type="text"
              placeholder="https://example.com/file.zip or magnet:..."
            />
          </label>
          <label>
            Referer (optional)
            <input v-model="newTaskReferer" type="url" placeholder="https://origin-page.example/" />
          </label>
          <div class="row">
            <button :disabled="creatingTask" type="submit">Add Task</button>
            <p class="status">{{ createTaskStatus }}</p>
          </div>
        </form>
        <p class="status">{{ taskStatus }}</p>
        <section class="task-list">
          <article v-for="task in tasks" :key="task.id" class="task">
            <h3>{{ task.spec.destination_path || task.id }}</h3>
            <p class="source">{{ task.spec.source.value }}</p>
            <p>
              <strong>{{ task.state }}</strong> · {{ progressText(task) }}
            </p>
            <div class="actions">
              <button v-if="canPause(task)" @click="action(task.id, 'pause')">Pause</button>
              <button v-else-if="canResume(task)" @click="action(task.id, 'resume')">Resume</button>
              <button class="danger" @click="action(task.id, 'remove')">Remove</button>
            </div>
          </article>
          <p v-if="!tasks.length">No tasks yet.</p>
        </section>
      </section>
    </div>

    <div v-if="removeDialogOpen" class="dialog-backdrop" @click.self="cancelRemoveDialog">
      <section class="dialog" role="dialog" aria-modal="true" aria-labelledby="remove-task-title">
        <h3 id="remove-task-title">Remove task?</h3>
        <p class="dialog-text">{{ taskTitle(removeDialogTaskId) }}</p>
        <label class="checkbox">
          <input v-model="removeDeleteFile" type="checkbox" />
          Delete downloaded file from disk
        </label>
        <div class="dialog-actions">
          <button @click="cancelRemoveDialog">Cancel</button>
          <button class="danger" @click="confirmRemoveDialog">Remove</button>
        </div>
      </section>
    </div>
  </main>
</template>

<style scoped>
.dashboard {
  max-width: 980px;
  margin: 18px auto;
  padding: 14px;
  color: #0f172a;
  font-family:
    ui-sans-serif,
    system-ui,
    -apple-system,
    sans-serif;
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

h1 {
  margin: 0;
}

h2 {
  margin: 0 0 8px;
}

button {
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  background: #fff;
  padding: 8px 10px;
  cursor: pointer;
}

.layout {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: 14px;
}

.card {
  border: 1px solid #e2e8f0;
  background: #f8fafc;
  border-radius: 10px;
  padding: 12px;
}

.settings {
  display: grid;
  gap: 10px;
  align-content: start;
}

label {
  display: grid;
  gap: 6px;
  font-size: 14px;
}

input[type='url'],
input[type='text'] {
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  padding: 10px;
}

.checkbox {
  grid-template-columns: auto 1fr;
  align-items: center;
  gap: 8px;
}

.row {
  display: grid;
  gap: 8px;
}

.status {
  margin: 0;
  color: #334155;
  min-height: 20px;
}

.task-list {
  display: grid;
  gap: 10px;
}

.add-task-form {
  display: grid;
  gap: 10px;
  margin-bottom: 10px;
}

.task {
  border: 1px solid #e2e8f0;
  background: #fff;
  border-radius: 10px;
  padding: 12px;
}

h3 {
  margin: 0 0 6px;
  font-size: 17px;
}

.source {
  margin: 0 0 8px;
  color: #475569;
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}

.danger {
  border-color: #fca5a5;
  color: #b91c1c;
}

.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: rgb(15 23 42 / 35%);
  display: grid;
  place-items: center;
  padding: 14px;
}

.dialog {
  width: min(420px, 100%);
  border: 1px solid #cbd5e1;
  background: #fff;
  border-radius: 10px;
  padding: 14px;
  display: grid;
  gap: 10px;
}

.dialog-text {
  margin: 0;
  color: #475569;
  font-size: 13px;
  word-break: break-all;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

@media (max-width: 860px) {
  .layout {
    grid-template-columns: 1fr;
  }
}
</style>
