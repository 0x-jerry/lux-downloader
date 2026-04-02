export interface LuxConfig {
  baseUrl: string
  authToken: string
  interceptEnabled: boolean
  includeCookies: boolean
  includeReferer: boolean
}

export interface LinkContext {
  url: string
  referer?: string
}

export interface RuntimeResponse<T = unknown> {
  ok: boolean
  data?: T
  error?: string
}

export interface TaskActionRequest {
  id: string
  action: 'pause' | 'resume' | 'restart' | 'remove'
  deleteFile?: boolean
}

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

export function filenameFromUrl(rawUrl: string): string {
  try {
    const url = new URL(rawUrl)
    const parts = url.pathname.split('/').filter(Boolean)
    const candidate = decodeURIComponent(parts[parts.length - 1] || '').trim()
    if (candidate) {
      return sanitizeFilename(candidate)
    }
  } catch {
    // ignore parse errors
  }

  return `download-${Date.now()}`
}

function sanitizeFilename(name: string): string {
  return (
    name
      .replace(/[\\/:*?"<>|]/g, '_')
      .replace(/\s+/g, ' ')
      .trim()
      .slice(0, 180) || `download-${Date.now()}`
  )
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

export function isLikelyDownloadLink(rawHref: string, hasDownloadAttribute: boolean): boolean {
  if (hasDownloadAttribute) {
    return true
  }

  const href = rawHref.toLowerCase()

  if (href.startsWith('magnet:?')) {
    return true
  }

  const knownSuffixes = [
    '.zip',
    '.7z',
    '.rar',
    '.tar',
    '.gz',
    '.bz2',
    '.xz',
    '.iso',
    '.dmg',
    '.pkg',
    '.exe',
    '.msi',
    '.deb',
    '.rpm',
    '.apk',
    '.mp4',
    '.mkv',
    '.avi',
    '.mp3',
    '.flac',
    '.wav',
    '.pdf',
    '.epub',
    '.torrent',
    '.metalink',
    '.meta4',
  ]

  const stripped = href.split('#')[0].split('?')[0]
  return knownSuffixes.some((suffix) => stripped.endsWith(suffix))
}

export async function validateConfig(config: LuxConfig): Promise<void> {
  const baseUrl = normalizeBaseUrl(config.baseUrl)

  const health = await fetch(`${baseUrl}/health`)
  if (!health.ok) {
    throw new Error(`Health check failed with ${health.status}`)
  }

  const tasks = await fetch(`${baseUrl}/tasks`, {
    headers: {
      Authorization: `Bearer ${config.authToken}`,
    },
  })

  if (!tasks.ok) {
    throw new Error(`Auth check failed with ${tasks.status}`)
  }
}

async function buildCookiePairs(rawUrl: string): Promise<Array<{ name: string; value: string }>> {
  const cookies = await browser.cookies.getAll({ url: rawUrl })
  return cookies.map((cookie) => ({ name: cookie.name, value: cookie.value }))
}

export async function createTaskFromLink(context: LinkContext): Promise<{ id: string }> {
  const config = await getConfig()
  const baseUrl = normalizeBaseUrl(config.baseUrl)

  if (!baseUrl) {
    throw new Error('Lux server URL is empty. Configure it in extension options.')
  }
  if (!config.authToken) {
    throw new Error('Lux auth token is empty. Configure it in extension options.')
  }

  const headers: Array<{ name: string; value: string }> = []
  if (config.includeReferer && context.referer) {
    headers.push({ name: 'Referer', value: context.referer })
  }
  headers.push({ name: 'User-Agent', value: navigator.userAgent })

  const cookies = config.includeCookies ? await buildCookiePairs(context.url) : []

  const payload = {
    source: {
      kind: 'auto',
      value: context.url,
    },
    destination_path: filenameFromUrl(context.url),
    settings: {
      headers,
      cookies,
    },
    auto_start: true,
  }

  const response = await fetch(`${baseUrl}/tasks`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${config.authToken}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(payload),
  })

  if (!response.ok) {
    let detail = `${response.status}`
    try {
      const body = await response.json()
      detail = String(body.error ?? detail)
    } catch {
      // keep status detail
    }
    throw new Error(`Failed to create task: ${detail}`)
  }

  const task = (await response.json()) as { id: string }
  return task
}

export async function listTasks(): Promise<unknown> {
  const config = await getConfig()
  const response = await fetch(`${normalizeBaseUrl(config.baseUrl)}/tasks`, {
    headers: {
      Authorization: `Bearer ${config.authToken}`,
    },
  })

  if (!response.ok) {
    throw new Error(`Failed to load tasks: ${response.status}`)
  }

  return response.json()
}

export async function taskAction(input: TaskActionRequest): Promise<unknown> {
  const config = await getConfig()
  const query =
    input.action === 'remove' && input.deleteFile ? '?delete_file=true' : ''
  const response = await fetch(
    `${normalizeBaseUrl(config.baseUrl)}/tasks/${input.id}/${input.action}${query}`,
    {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${config.authToken}`,
      },
    },
  )

  if (!response.ok) {
    throw new Error(`Failed to ${input.action}: ${response.status}`)
  }

  return response.json()
}

export async function getTorrentStats(taskId: string): Promise<unknown> {
  const config = await getConfig()
  const response = await fetch(
    `${normalizeBaseUrl(config.baseUrl)}/tasks/${taskId}/torrent-stats`,
    {
      headers: {
        Authorization: `Bearer ${config.authToken}`,
      },
    },
  )

  if (!response.ok) {
    throw new Error(`Failed to load torrent details: ${response.status}`)
  }

  return response.json()
}

export async function checkServerHealth(): Promise<{ connected: boolean }> {
  const config = await getConfig()
  const response = await fetch(`${normalizeBaseUrl(config.baseUrl)}/health`)
  return { connected: response.ok }
}
