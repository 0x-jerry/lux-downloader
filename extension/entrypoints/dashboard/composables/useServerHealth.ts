import { reactive } from 'vue'

export type ServerConnection = 'checking' | 'connected' | 'disconnected'

export function useServerHealth() {
  const state = reactive({
    connection: 'checking' as ServerConnection,
  })

  async function refresh() {
    try {
      const response = await browser.runtime.sendMessage({ action: 'server_health' })
      state.connection = response?.ok && response.data?.connected ? 'connected' : 'disconnected'
    } catch {
      state.connection = 'disconnected'
    }
  }

  return { state, refresh }
}
