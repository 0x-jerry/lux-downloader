import type { DashboardState } from '../dashboardState'

type ServerHealthModuleParams = {
  state: DashboardState
}

export function useServerHealth({ state }: ServerHealthModuleParams) {
  async function refreshServerHealth() {
    try {
      const response = await browser.runtime.sendMessage({ action: 'server_health' })
      if (!response?.ok) {
        state.serverConnection = 'disconnected'
        return
      }

      const connected = Boolean(response.data?.connected)
      state.serverConnection = connected ? 'connected' : 'disconnected'
    } catch {
      state.serverConnection = 'disconnected'
    }
  }

  return {
    refreshServerHealth,
  }
}
