import { reactive } from 'vue'
import type { DashboardState } from './dashboardState'
import { useConfigSettings } from './modules/useConfigSettings'
import { useServerHealth } from './modules/useServerHealth'
import { useTaskManagement } from './modules/useTaskManagement'
import { useTorrentDetails } from './modules/useTorrentDetails'

export function useDashboard() {
  const state = reactive<DashboardState>({
    tasks: [],
    taskStatus: '',
    createTaskStatus: '',
    creatingTask: false,
    configStatus: '',
    savingConfig: false,
    serverConnection: 'checking',
    removeDialogOpen: false,
    removeDialogTaskId: null,
    removeDeleteFile: false,
    torrentDialogOpen: false,
    torrentDialogTaskId: null,
    torrentDetails: {},
    newTaskUrl: '',
    newTaskReferer: '',
    form: {
      baseUrl: 'http://127.0.0.1:8080',
      authToken: 'change-me',
      interceptEnabled: true,
      includeCookies: true,
      includeReferer: true,
    },
  })

  const configSettings = useConfigSettings({
    state,
  })

  const serverHealth = useServerHealth({
    state,
  })

  const torrentDetailState = useTorrentDetails({
    state,
  })

  const taskManagement = useTaskManagement({
    state,
    closeTorrentDetails: torrentDetailState.closeTorrentDetails,
    reconcileWithActiveTasks: torrentDetailState.reconcileWithActiveTasks,
    refreshActiveDialogDetails: torrentDetailState.refreshActiveDialogDetails,
  })

  let taskTimer: number | undefined
  let serverTimer: number | undefined

  async function init() {
    await configSettings.loadConfig()
    await Promise.all([taskManagement.loadTasks(), serverHealth.refreshServerHealth()])
    taskTimer = window.setInterval(() => {
      void taskManagement.loadTasks()
    }, 3000)
    serverTimer = window.setInterval(() => {
      void serverHealth.refreshServerHealth()
    }, 5000)
  }

  function dispose() {
    if (taskTimer) {
      window.clearInterval(taskTimer)
    }
    if (serverTimer) {
      window.clearInterval(serverTimer)
    }
  }

  function taskTitle(taskId: string | null): string {
    if (!taskId) {
      return ''
    }

    const task = state.tasks.find((item) => item.id === taskId)
    return task?.spec.destination_path || taskId
  }

  return {
    state,
    init,
    dispose,
    loadTasks: taskManagement.loadTasks,
    createTask: taskManagement.createTask,
    action: taskManagement.action,
    saveConfig: configSettings.saveConfig,
    refreshServerHealth: serverHealth.refreshServerHealth,
    cancelRemoveDialog: taskManagement.cancelRemoveDialog,
    confirmRemoveDialog: taskManagement.confirmRemoveDialog,
    taskTitle,
    openTorrentDetails: torrentDetailState.openTorrentDetails,
    closeTorrentDetails: torrentDetailState.closeTorrentDetails,
    refreshCurrentTorrentDetails: torrentDetailState.refreshCurrentTorrentDetails,
  }
}
