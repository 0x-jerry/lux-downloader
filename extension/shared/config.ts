import type { LuxConfig } from './types'

const DEFAULT_CONFIG: LuxConfig = {
  baseUrl: 'http://127.0.0.1:8080',
  authToken: 'change-me',
  interceptEnabled: true,
  includeCookies: true,
  includeReferer: true,
}

export function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.trim().replace(/\/+$/, '')
}

export async function getConfig(): Promise<LuxConfig> {
  const stored = await browser.storage.sync.get(Object.keys(DEFAULT_CONFIG))
  return {
    baseUrl: normalizeBaseUrl(String(stored.baseUrl ?? DEFAULT_CONFIG.baseUrl)),
    authToken: String(stored.authToken ?? DEFAULT_CONFIG.authToken),
    interceptEnabled: Boolean(stored.interceptEnabled ?? DEFAULT_CONFIG.interceptEnabled),
    includeCookies: Boolean(stored.includeCookies ?? DEFAULT_CONFIG.includeCookies),
    includeReferer: Boolean(stored.includeReferer ?? DEFAULT_CONFIG.includeReferer),
  }
}

export async function saveConfig(input: Partial<LuxConfig>): Promise<LuxConfig> {
  const current = await getConfig()
  const next: LuxConfig = {
    ...current,
    ...input,
    baseUrl: normalizeBaseUrl(input.baseUrl ?? current.baseUrl),
  }

  await browser.storage.sync.set(next)
  return next
}
