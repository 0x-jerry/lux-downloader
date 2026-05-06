<script setup lang="ts">
import { computed, onMounted, onUnmounted, provide } from 'vue'
import SettingsCard from './components/SettingsCard.vue'
import TaskCreateForm from './components/TaskCreateForm.vue'
import TaskList from './components/TaskList.vue'
import { TASKS_KEY, useTasks } from './composables/useTasks'
import { useServerHealth } from './composables/useServerHealth'

const tasksService = useTasks()
provide(TASKS_KEY, tasksService)

const serverHealth = useServerHealth()

const serverStatusLabel = computed(() => {
  switch (serverHealth.state.connection) {
    case 'connected':
      return 'Server Connected'
    case 'disconnected':
      return 'Server Disconnected'
    default:
      return 'Checking Server'
  }
})

const serverStatusTheme = computed(() => {
  switch (serverHealth.state.connection) {
    case 'connected':
      return 'success'
    case 'disconnected':
      return 'danger'
    default:
      return 'warning'
  }
})

let taskTimer: number | undefined
let serverTimer: number | undefined

onMounted(async () => {
  await tasksService.loadTasks()
  await serverHealth.refresh()
  taskTimer = window.setInterval(() => {
    void tasksService.loadTasks()
  }, 3000)
  serverTimer = window.setInterval(() => {
    void serverHealth.refresh()
  }, 5000)
})

onUnmounted(() => {
  if (taskTimer) window.clearInterval(taskTimer)
  if (serverTimer) window.clearInterval(serverTimer)
})

function refreshAll() {
  void Promise.all([tasksService.loadTasks(), serverHealth.refresh()])
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
      <SettingsCard />

      <t-card title="Tasks" bordered>
        <TaskCreateForm />
        <p class="status">{{ tasksService.state.status }}</p>
        <TaskList />
      </t-card>
    </div>

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
