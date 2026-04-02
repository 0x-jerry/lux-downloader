import type { Task } from './types'

export function formatBytes(bytes: number): string {
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

export function progressText(task: Task): string {
  const downloaded = formatBytes(task.progress.downloaded_bytes)
  const total = task.progress.total_bytes ? formatBytes(task.progress.total_bytes) : '?'
  const downRate = formatBytes(task.progress.download_rate_bps) + '/s'
  const upRate = formatBytes(task.progress.upload_rate_bps) + '/s'
  return `${downloaded} / ${total} (Down ${downRate}, Up ${upRate})`
}

export function canPause(task: Task): boolean {
  const state = task.state.toLowerCase()
  return ['queued', 'metadata_fetching', 'downloading', 'seeding'].includes(state)
}

export function canResume(task: Task): boolean {
  return task.state.toLowerCase() === 'paused'
}

export function canRestart(task: Task): boolean {
  return task.state.toLowerCase() === 'failed'
}

export function isTorrentTask(task: Task): boolean {
  return task.spec.source.kind === 'torrent' || task.spec.source.kind === 'magnet'
}
