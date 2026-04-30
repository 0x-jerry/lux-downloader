import type { LuxConfig } from '../../../shared'
import type { Task, TorrentDetailEntry } from '../types'

export type ServerConnection = 'checking' | 'connected' | 'disconnected'

export type DashboardState = {
  tasks: Task[]
  taskStatus: string
  createTaskStatus: string
  creatingTask: boolean
  configStatus: string
  savingConfig: boolean
  serverConnection: ServerConnection
  removeDialogOpen: boolean
  removeDialogTaskId: string | null
  removeDeleteFile: boolean
  torrentDialogOpen: boolean
  torrentDialogTaskId: string | null
  torrentDetails: Record<string, TorrentDetailEntry>
  newTaskUrl: string
  newTaskReferer: string
  form: LuxConfig
  retryDialogOpen: boolean
  retryDialogError: string
  retryDialogUrl: string
  retryDialogReferer: string
  retryDialogFilename: string
  retryDialogOverwrite: boolean
}
