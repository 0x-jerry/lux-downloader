import type { Task, TorrentDetailEntry, TorrentStats } from '../../types'
import { isTorrentTask } from '../../utils'
import type { DashboardState } from '../dashboardState'

type TorrentDetailsModuleParams = {
  state: DashboardState
}

export function useTorrentDetails({
  state,
}: TorrentDetailsModuleParams) {
  function ensureTorrentDetailEntry(taskId: string): TorrentDetailEntry {
    if (!state.torrentDetails[taskId]) {
      state.torrentDetails[taskId] = {
        loading: false,
        error: '',
        data: null,
      }
    }
    return state.torrentDetails[taskId]
  }

  async function loadTorrentDetails(taskId: string, setStatusOnFailure: boolean) {
    const entry = ensureTorrentDetailEntry(taskId)
    entry.loading = true
    entry.error = ''

    try {
      const response = await browser.runtime.sendMessage({
        action: 'torrent_stats',
        payload: { id: taskId },
      })

      if (!response?.ok) {
        const message = response?.error ?? 'Failed to load torrent details'
        entry.error = message
        if (setStatusOnFailure) {
          state.taskStatus = message
        }
        return
      }

      entry.data = response.data as TorrentStats
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      entry.error = message
      if (setStatusOnFailure) {
        state.taskStatus = message
      }
    } finally {
      entry.loading = false
    }
  }

  async function openTorrentDetails(task: Task) {
    if (!isTorrentTask(task)) {
      return
    }

    state.torrentDialogTaskId = task.id
    state.torrentDialogOpen = true
    await loadTorrentDetails(task.id, true)
  }

  function closeTorrentDetails() {
    state.torrentDialogOpen = false
    state.torrentDialogTaskId = null
  }

  async function refreshCurrentTorrentDetails() {
    const taskId = state.torrentDialogTaskId
    if (!taskId) {
      return
    }
    await loadTorrentDetails(taskId, true)
  }

  function reconcileWithActiveTasks(activeIds: Set<string>) {
    Object.keys(state.torrentDetails).forEach((taskId) => {
      if (!activeIds.has(taskId)) {
        delete state.torrentDetails[taskId]
      }
    })
    if (state.torrentDialogTaskId && !activeIds.has(state.torrentDialogTaskId)) {
      closeTorrentDetails()
    }
  }

  async function refreshActiveDialogDetails() {
    if (state.torrentDialogOpen && state.torrentDialogTaskId) {
      await loadTorrentDetails(state.torrentDialogTaskId, false)
    }
  }

  return {
    loadTorrentDetails,
    openTorrentDetails,
    closeTorrentDetails,
    refreshCurrentTorrentDetails,
    reconcileWithActiveTasks,
    refreshActiveDialogDetails,
  }
}
