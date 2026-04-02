import {
  checkServerHealth,
  createTaskFromLink,
  getTorrentStats,
  getConfig,
  listTasks,
  saveConfig,
  taskAction,
  validateConfig,
  type LinkContext,
  type LuxConfig,
  type RuntimeResponse,
  type TaskActionRequest,
} from '../src/shared'

const MENU_ID = 'lux-send-link'

export default defineBackground(() => {
  browser.runtime.onInstalled.addListener(() => {
    browser.contextMenus.create({
      id: MENU_ID,
      title: 'Send link to Lux',
      contexts: ['link'],
    })
  })

  browser.contextMenus.onClicked.addListener(async (info) => {
    if (info.menuItemId !== MENU_ID || !info.linkUrl) {
      return
    }

    try {
      await createTaskFromLink({
        url: info.linkUrl,
        referer: info.pageUrl,
      })
    } catch (error) {
      console.error('Failed to create Lux task from context menu:', error)
    }
  })

  browser.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    void handleMessage(message)
      .then((response) => sendResponse(response))
      .catch((error) =>
        sendResponse({
          ok: false,
          error: error instanceof Error ? error.message : String(error),
        } satisfies RuntimeResponse),
      )

    // Keep the message channel open for async responses.
    return true
  })
})

async function handleMessage(message: unknown): Promise<RuntimeResponse> {
  if (!message || typeof message !== 'object') {
    return { ok: false, error: 'Invalid message payload' }
  }

  const request = message as Record<string, unknown>

  switch (request.action) {
    case 'intercept_add_task': {
      const config = await getConfig()
      if (!config.interceptEnabled) {
        return { ok: false, error: 'Interception is disabled in options' }
      }

      const context = request.payload as LinkContext
      const task = await createTaskFromLink(context)
      return { ok: true, data: task }
    }

    case 'manual_add_task': {
      const context = request.payload as LinkContext
      const task = await createTaskFromLink(context)
      return { ok: true, data: task }
    }

    case 'list_tasks': {
      const tasks = await listTasks()
      return { ok: true, data: tasks }
    }

    case 'task_action': {
      const payload = request.payload as TaskActionRequest
      const task = await taskAction(payload)
      return { ok: true, data: task }
    }

    case 'torrent_stats': {
      const payload = request.payload as { id: string }
      const stats = await getTorrentStats(payload.id)
      return { ok: true, data: stats }
    }

    case 'server_health': {
      const health = await checkServerHealth()
      return { ok: true, data: health }
    }

    case 'get_config': {
      const config = await getConfig()
      return { ok: true, data: config }
    }

    case 'save_config': {
      const payload = request.payload as Partial<LuxConfig>
      const config = await saveConfig(payload)
      return { ok: true, data: config }
    }

    case 'validate_config': {
      const payload = request.payload as LuxConfig
      await validateConfig(payload)
      return { ok: true }
    }

    default:
      return { ok: false, error: `Unknown action: ${String(request.action)}` }
  }
}
