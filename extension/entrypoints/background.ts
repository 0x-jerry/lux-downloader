import {
  checkServerHealth,
  createTaskFromLink,
  getTorrentStats,
  getConfig,
  isLikelyDownloadLink,
  listTasks,
  patchTask,
  saveConfig,
  taskAction,
  validateConfig,
  type LinkContext,
  type LuxConfig,
  type TaskPatchRequest,
  type RuntimeResponse,
  type TaskActionRequest,
  type Task,
} from '../shared'

type MessagePayload =
  | { action: 'intercept_add_task'; payload: LinkContext }
  | { action: 'manual_add_task'; payload: LinkContext }
  | { action: 'list_tasks' }
  | { action: 'task_action'; payload: TaskActionRequest }
  | { action: 'patch_task'; payload: { id: string; patch: TaskPatchRequest } }
  | { action: 'torrent_stats'; payload: { id: string } }
  | { action: 'server_health' }
  | { action: 'get_config' }
  | { action: 'save_config'; payload: Partial<LuxConfig> }
  | { action: 'validate_config'; payload: LuxConfig }

const MENU_ID = 'lux-send-link'
const BADGE_ALARM = 'update-badge'

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
      updateBadge()
    } catch (error) {
      console.error('Failed to create Lux task from context menu:', error)
    }
  })

  async function updateBadge() {
    const tasks = (await listTasks()).items

    const count = tasks.filter((t) => t.state === 'downloading').length

    if (count > 0) {
      await browser.action.setBadgeText({ text: String(count) })
      await browser.action.setBadgeBackgroundColor({ color: '#1a73e8' })
    } else {
      await browser.action.setBadgeText({ text: '' })
    }
  }

  browser.alarms.onAlarm.addListener((alarm) => {
    if (alarm.name === BADGE_ALARM) {
      updateBadge()
    }
  })

  browser.alarms.create(BADGE_ALARM, { periodInMinutes: 1 / 12 })

  updateBadge()

  browser.downloads.onDeterminingFilename.addListener((downloadItem, suggest) => {
    const { url, referrer } = downloadItem

    if (!url || url.startsWith('blob:') || url.startsWith('data:')) {
      suggest()
      return
    }

    ;(async () => {
      const config = await getConfig()
      if (!config.interceptEnabled || !isLikelyDownloadLink(url, false)) {
        suggest()
        return
      }

      try {
        const task = await createTaskFromLink({
          url,
          referer: referrer ?? undefined,
        })

        await updateBadge()
        await browser.downloads.cancel(downloadItem.id)
        console.log('Download intercepted and sent to Lux:', task.id)
      } catch (error) {
        console.error('Failed to intercept download:', error)
      }

      suggest()
    })()

    return true
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

function isValidMessage(msg: unknown): msg is MessagePayload {
  return msg !== null && typeof msg === 'object' && 'action' in msg
}

async function handleMessage(message: unknown): Promise<RuntimeResponse> {
  if (!isValidMessage(message)) {
    return { ok: false, error: 'Invalid message payload' }
  }

  const request = message

  switch (request.action) {
    case 'intercept_add_task': {
      const config = await getConfig()
      if (!config.interceptEnabled) {
        return { ok: false, error: 'Interception is disabled in options' }
      }

      const task = await createTaskFromLink(request.payload)
      return { ok: true, data: task }
    }

    case 'manual_add_task': {
      const task = await createTaskFromLink(request.payload)
      return { ok: true, data: task }
    }

    case 'list_tasks': {
      const tasks = await listTasks()
      return { ok: true, data: tasks }
    }

    case 'task_action': {
      const task = await taskAction(request.payload)
      return { ok: true, data: task }
    }

    case 'patch_task': {
      const task = await patchTask(request.payload.id, request.payload.patch)
      return { ok: true, data: task }
    }

    case 'torrent_stats': {
      const stats = await getTorrentStats(request.payload.id)
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
      const config = await saveConfig(request.payload)
      return { ok: true, data: config }
    }

    case 'validate_config': {
      await validateConfig(request.payload)
      return { ok: true }
    }
  }
}
