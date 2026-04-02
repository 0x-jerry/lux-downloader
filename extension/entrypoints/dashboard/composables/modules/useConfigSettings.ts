import type { DashboardState } from '../dashboardState'

type ConfigModuleParams = {
  state: DashboardState
}

export function useConfigSettings({ state }: ConfigModuleParams) {
  async function loadConfig() {
    try {
      const response = await browser.runtime.sendMessage({ action: 'get_config' })
      if (!response?.ok) {
        state.configStatus = response?.error ?? 'Failed to load config'
        return
      }

      Object.assign(state.form, response.data)
    } catch (error) {
      state.configStatus = error instanceof Error ? error.message : String(error)
    }
  }

  async function saveConfig() {
    state.savingConfig = true
    state.configStatus = 'Saving...'

    try {
      const saved = await browser.runtime.sendMessage({
        action: 'save_config',
        payload: state.form,
      })

      if (!saved?.ok) {
        state.configStatus = saved?.error ?? 'Failed to save config'
        return
      }

      const validation = await browser.runtime.sendMessage({
        action: 'validate_config',
        payload: state.form,
      })

      if (!validation?.ok) {
        state.configStatus = `Saved, but validation failed: ${validation.error}`
        return
      }

      state.configStatus = 'Saved and validated successfully.'
    } finally {
      state.savingConfig = false
    }
  }

  return {
    loadConfig,
    saveConfig,
  }
}
