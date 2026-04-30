import type { LinkContext, LuxConfig, Task, TaskActionRequest, TaskPatchRequest, ListTasksResponse, TorrentStats } from './types'
import { getConfig, normalizeBaseUrl } from './config'
import { filenameFromUrl } from './utils'

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
      kind: 'auto' as const,
      value: context.url,
    },
    destination_path: context.destinationPath ?? filenameFromUrl(context.url),
    settings: {
      headers,
      cookies,
    },
    auto_start: true,
    ...(context.overwrite ? { overwrite: true } : {}),
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

export async function listTasks(): Promise<ListTasksResponse> {
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

export async function taskAction(input: TaskActionRequest): Promise<Task> {
  const config = await getConfig()
  const query = input.action === 'remove' && input.deleteFile ? '?delete_file=true' : ''
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

export async function patchTask(id: string, patch: TaskPatchRequest): Promise<Task> {
  const config = await getConfig()
  const response = await fetch(`${normalizeBaseUrl(config.baseUrl)}/tasks/${id}`, {
    method: 'PATCH',
    headers: {
      Authorization: `Bearer ${config.authToken}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(patch),
  })

  if (!response.ok) {
    throw new Error(`Failed to patch task: ${response.status}`)
  }

  return response.json()
}

export async function getTorrentStats(taskId: string): Promise<TorrentStats> {
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
