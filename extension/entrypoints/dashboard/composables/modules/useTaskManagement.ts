import type { Task } from '../../types'
import type { DashboardState } from '../dashboardState'

type TaskCommand = 'pause' | 'resume' | 'restart' | 'remove'

type TaskManagementModuleParams = {
  state: DashboardState
  closeTorrentDetails: () => void
  reconcileWithActiveTasks: (activeIds: Set<string>) => void
  refreshActiveDialogDetails: () => Promise<void>
}

export function useTaskManagement({
  state,
  closeTorrentDetails,
  reconcileWithActiveTasks,
  refreshActiveDialogDetails,
}: TaskManagementModuleParams) {
  async function loadTasks() {
    try {
      const response = await browser.runtime.sendMessage({ action: 'list_tasks' })
      if (!response?.ok) {
        state.taskStatus = response?.error ?? 'Failed to load tasks'
        return
      }

      state.tasks = (response.data?.items ?? []) as Task[]
      state.taskStatus = `Loaded ${state.tasks.length} tasks`

      const activeIds = new Set(state.tasks.map((task) => task.id))
      reconcileWithActiveTasks(activeIds)
      await refreshActiveDialogDetails()
    } catch (error) {
      state.taskStatus = error instanceof Error ? error.message : String(error)
    }
  }

  async function createTask() {
    if (!state.newTaskUrl.trim()) {
      state.createTaskStatus = 'Enter a URL first.'
      return
    }

    state.creatingTask = true
    state.createTaskStatus = 'Creating task...'

    try {
      const response = await browser.runtime.sendMessage({
        action: 'manual_add_task',
        payload: {
          url: state.newTaskUrl.trim(),
          referer: state.newTaskReferer.trim(),
        },
      })

      if (!response?.ok) {
        state.createTaskStatus = response?.error ?? 'Failed to create task'
        return
      }

      state.createTaskStatus = `Task created: ${response.data.id}`
      state.newTaskUrl = ''
      state.newTaskReferer = ''
      await loadTasks()
    } finally {
      state.creatingTask = false
    }
  }

  async function action(id: string, command: TaskCommand) {
    if (command === 'remove') {
      openRemoveDialog(id)
      return
    }

    await runTaskAction(id, command)
  }

  async function runTaskAction(id: string, command: TaskCommand, deleteFile?: boolean) {
    const response = await browser.runtime.sendMessage({
      action: 'task_action',
      payload: { id, action: command, deleteFile },
    })

    if (!response?.ok) {
      state.taskStatus = response?.error ?? `Failed to ${command}`
      return
    }

    state.taskStatus = `${command} succeeded`
    if (command === 'remove') {
      delete state.torrentDetails[id]
      if (state.torrentDialogTaskId === id) {
        closeTorrentDetails()
      }
    }
    await loadTasks()
  }

  function openRemoveDialog(id: string) {
    state.removeDialogTaskId = id
    state.removeDeleteFile = false
    state.removeDialogOpen = true
  }

  function cancelRemoveDialog() {
    state.removeDialogOpen = false
    state.removeDialogTaskId = null
    state.removeDeleteFile = false
  }

  async function confirmRemoveDialog() {
    const id = state.removeDialogTaskId
    if (!id) {
      return
    }

    const deleteFile = state.removeDeleteFile
    cancelRemoveDialog()
    await runTaskAction(id, 'remove', deleteFile)
  }

  return {
    loadTasks,
    createTask,
    action,
    cancelRemoveDialog,
    confirmRemoveDialog,
  }
}
