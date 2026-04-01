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
  }
  spec: {
    source: { value: string }
    destination_path: string
  }
}

const tasks = ref<Task[]>([])
const taskStatus = ref('')
const configStatus = ref('')
const savingConfig = ref(false)
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

async function action(id: string, command: 'pause' | 'resume' | 'remove') {
  const response = await browser.runtime.sendMessage({
    action: 'task_action',
    payload: { id, action: command },
  })

  if (!response?.ok) {
    taskStatus.value = response?.error ?? `Failed to ${command}`
    return
  }

  taskStatus.value = `${command} succeeded`
  await loadTasks()
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
  const rate = formatBytes(task.progress.download_rate_bps) + '/s'
  return `${downloaded} / ${total} (${rate})`
}

function canPause(task: Task): boolean {
  const state = task.state.toLowerCase()
  return ['queued', 'metadata_fetching', 'downloading', 'seeding'].includes(state)
}

function canResume(task: Task): boolean {
  return task.state.toLowerCase() === 'paused'
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

@media (max-width: 860px) {
  .layout {
    grid-template-columns: 1fr;
  }
}
</style>
