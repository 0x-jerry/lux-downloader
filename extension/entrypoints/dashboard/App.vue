<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import type { Task } from './types'
import ChangeSourceDialog from './components/ChangeSourceDialog.vue'
import RemoveTaskDialog from './components/RemoveTaskDialog.vue'
import SettingsCard from './components/SettingsCard.vue'
import TorrentDetailDialog from './components/TorrentDetailDialog.vue'
import TaskCreateForm from './components/TaskCreateForm.vue'
import TaskList from './components/TaskList.vue'
import { useDashboard } from './composables/useDashboard'

const dashboard = useDashboard()

const removeTaskTitle = computed(() => dashboard.taskTitle(dashboard.state.removeDialogTaskId))
const torrentTaskTitle = computed(() => dashboard.taskTitle(dashboard.state.torrentDialogTaskId))
const sourceDialogOpen = ref(false)
const sourceDialogTaskId = ref<string | null>(null)
const sourceDialogValue = ref('')
const sourceDialogTaskTitle = computed(() => dashboard.taskTitle(sourceDialogTaskId.value))
const activeTorrentDetail = computed(() => {
  const taskId = dashboard.state.torrentDialogTaskId
  if (!taskId) {
    return null
  }
  return dashboard.state.torrentDetails[taskId] ?? null
})
const serverStatusLabel = computed(() => {
  switch (dashboard.state.serverConnection) {
    case 'connected':
      return 'Server Connected'
    case 'disconnected':
      return 'Server Disconnected'
    default:
      return 'Checking Server'
  }
})

const serverStatusTheme = computed(() => {
  switch (dashboard.state.serverConnection) {
    case 'connected':
      return 'success'
    case 'disconnected':
      return 'danger'
    default:
      return 'warning'
  }
})

onMounted(async () => {
  await dashboard.init()
})

onUnmounted(() => {
  dashboard.dispose()
})

function refreshAll() {
  void Promise.all([dashboard.loadTasks(), dashboard.refreshServerHealth()])
}

function openChangeSourceDialog(task: Task) {
  sourceDialogTaskId.value = task.id
  sourceDialogValue.value = task.spec.source.value
  sourceDialogOpen.value = true
}

function closeChangeSourceDialog() {
  sourceDialogOpen.value = false
  sourceDialogTaskId.value = null
  sourceDialogValue.value = ''
}

async function confirmChangeSourceDialog() {
  const taskId = sourceDialogTaskId.value
  if (!taskId) {
    return
  }

  const success = await dashboard.updateTaskSource(taskId, sourceDialogValue.value)
  if (success) {
    closeChangeSourceDialog()
  }
}
</script>

<template>
  <main class="dashboard">
    <header class="header">
      <div class="header-title">
        <h1>Lux Dashboard</h1>
        <t-tag variant="light-outline" :theme="serverStatusTheme">
          {{ serverStatusLabel }}
        </t-tag>
      </div>
      <t-button variant="outline" @click="refreshAll">Refresh Tasks</t-button>
    </header>

    <div class="layout">
      <SettingsCard
        :form="dashboard.state.form"
        :saving-config="dashboard.state.savingConfig"
        :config-status="dashboard.state.configStatus"
        @save="dashboard.saveConfig"
      />

      <t-card title="Tasks" bordered>
        <TaskCreateForm
          :new-task-url="dashboard.state.newTaskUrl"
          :new-task-referer="dashboard.state.newTaskReferer"
          :creating-task="dashboard.state.creatingTask"
          :create-task-status="dashboard.state.createTaskStatus"
          @submit="dashboard.createTask"
          @update:new-task-url="dashboard.state.newTaskUrl = $event"
          @update:new-task-referer="dashboard.state.newTaskReferer = $event"
        />
        <p class="status">{{ dashboard.state.taskStatus }}</p>
        <TaskList
          :tasks="dashboard.state.tasks"
          @action="dashboard.action"
          @open-change-source="openChangeSourceDialog"
          @open-torrent-details="dashboard.openTorrentDetails"
        />
      </t-card>
    </div>

    <ChangeSourceDialog
      :open="sourceDialogOpen"
      :task-title="sourceDialogTaskTitle"
      :value="sourceDialogValue"
      @cancel="closeChangeSourceDialog"
      @confirm="confirmChangeSourceDialog"
      @update:value="sourceDialogValue = $event"
    />

    <TorrentDetailDialog
      :open="dashboard.state.torrentDialogOpen"
      :task-title="torrentTaskTitle"
      :detail="activeTorrentDetail"
      @close="dashboard.closeTorrentDetails"
      @refresh="dashboard.refreshCurrentTorrentDetails"
    />

    <RemoveTaskDialog
      :open="dashboard.state.removeDialogOpen"
      :task-title="removeTaskTitle"
      :remove-delete-file="dashboard.state.removeDeleteFile"
      @cancel="dashboard.cancelRemoveDialog"
      @confirm="dashboard.confirmRemoveDialog"
      @update:remove-delete-file="dashboard.state.removeDeleteFile = $event"
    />
  </main>
</template>

<style scoped>
.dashboard {
  margin: 18px auto;
  padding: 14px;
  color: #0f172a;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  gap: 12px;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

h1 {
  margin: 0;
}

.layout {
  display: grid;
  grid-template-columns: 350px 1fr;
  gap: 14px;
}

.status {
  margin: 0 0 10px;
  color: #334155;
  min-height: 20px;
}

@media (max-width: 860px) {
  .layout {
    grid-template-columns: 1fr;
  }

  .header {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
