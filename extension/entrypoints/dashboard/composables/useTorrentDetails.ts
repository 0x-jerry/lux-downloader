import { reactive } from 'vue'
import type { TorrentStats } from '../types'

export interface TorrentDetailEntry {
  loading: boolean
  error: string
  data: TorrentStats | null
}

export function useTorrentDetails() {
  const cache = reactive<Record<string, TorrentDetailEntry>>({})

  function ensure(taskId: string): TorrentDetailEntry {
    if (!cache[taskId]) {
      cache[taskId] = { loading: false, error: '', data: null }
    }
    return cache[taskId]
  }

  async function fetch(taskId: string) {
    const entry = ensure(taskId)
    entry.loading = true
    entry.error = ''

    try {
      const response = await browser.runtime.sendMessage({
        action: 'torrent_stats',
        payload: { id: taskId },
      })

      if (!response?.ok) {
        entry.error = response?.error ?? 'Failed to load torrent details'
        return
      }

      entry.data = response.data as TorrentStats
    } catch (error) {
      entry.error = error instanceof Error ? error.message : String(error)
    } finally {
      entry.loading = false
    }
  }

  function reconcile(activeIds: Set<string>) {
    Object.keys(cache).forEach((id) => {
      if (!activeIds.has(id)) delete cache[id]
    })
  }

  function evict(taskId: string) {
    delete cache[taskId]
  }

  return { cache, fetch, reconcile, evict }
}
