import type { LuxConfig } from '../../../shared'

export function useConfig() {
  async function load(): Promise<LuxConfig> {
    const response = await browser.runtime.sendMessage({ action: 'get_config' })
    if (!response?.ok) {
      throw new Error(response?.error ?? 'Failed to load config')
    }
    return response.data as LuxConfig
  }

  async function save(config: LuxConfig) {
    const saved = await browser.runtime.sendMessage({
      action: 'save_config',
      payload: config,
    })
    if (!saved?.ok) {
      return { ok: false as const, error: saved?.error ?? 'Failed to save config' }
    }

    const validation = await browser.runtime.sendMessage({
      action: 'validate_config',
      payload: config,
    })
    if (!validation?.ok) {
      return { ok: false as const, error: `Saved, but validation failed: ${validation.error}` }
    }

    return { ok: true as const }
  }

  return { load, save }
}
