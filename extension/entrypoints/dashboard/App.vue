<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import RemoveTaskDialog from './components/RemoveTaskDialog.vue'
import SettingsCard from './components/SettingsCard.vue'
import TorrentDetailDialog from './components/TorrentDetailDialog.vue'
import TaskCreateForm from './components/TaskCreateForm.vue'
import TaskList from './components/TaskList.vue'
import { useDashboard } from './composables/useDashboard'

const dashboard = useDashboard()

const removeTaskTitle = computed(() => dashboard.taskTitle(dashboard.state.removeDialogTaskId))
const torrentTaskTitle = computed(() => dashboard.taskTitle(dashboard.state.torrentDialogTaskId))
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

onMounted(async () => {
  await dashboard.init()
})

onUnmounted(() => {
  dashboard.dispose()
})

function refreshAll() {
  void Promise.all([dashboard.loadTasks(), dashboard.refreshServerHealth()])
}
</script>

<template>
  <main class="dashboard">
    <header>
      <div class="header-title">
        <h1>Lux Dashboard</h1>
        <span class="badge" :class="dashboard.state.serverConnection">{{ serverStatusLabel }}</span>
      </div>
      <button @click="refreshAll">Refresh Tasks</button>
    </header>

    <div class="layout">
      <SettingsCard
        :form="dashboard.state.form"
        :saving-config="dashboard.state.savingConfig"
        :config-status="dashboard.state.configStatus"
        @save="dashboard.saveConfig"
      />

      <section class="card tasks">
        <h2>Tasks</h2>
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
          @open-torrent-details="dashboard.openTorrentDetails"
        />
      </section>
    </div>

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

<style>
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

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
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

.badge {
  border-radius: 999px;
  padding: 4px 10px;
  font-size: 12px;
  line-height: 1.2;
  border: 1px solid #cbd5e1;
  background: #f1f5f9;
  color: #334155;
}

.badge.connected {
  border-color: #86efac;
  background: #dcfce7;
  color: #166534;
}

.badge.disconnected {
  border-color: #fca5a5;
  background: #fee2e2;
  color: #b91c1c;
}

.badge.checking {
  border-color: #fcd34d;
  background: #fef3c7;
  color: #92400e;
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

.status {
  margin: 0;
  color: #334155;
  min-height: 20px;
}

@media (max-width: 860px) {
  .layout {
    grid-template-columns: 1fr;
  }
}
</style>
