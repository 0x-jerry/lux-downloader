import { isLikelyDownloadLink } from '../src/shared'

export default defineContentScript({
  matches: ['<all_urls>'],
  runAt: 'document_start',
  main() {
    document.addEventListener(
      'click',
      (event) => {
        if (event.defaultPrevented || event.button !== 0) {
          return
        }

        const target = event.target as HTMLElement | null
        const anchor = target?.closest('a[href]') as HTMLAnchorElement | null
        if (!anchor) {
          return
        }

        const href = anchor.href
        if (!href) {
          return
        }

        const isDownload = isLikelyDownloadLink(href, anchor.hasAttribute('download'))
        if (!isDownload) {
          return
        }

        event.preventDefault()
        event.stopPropagation()

        void browser.runtime
          .sendMessage({
            action: 'intercept_add_task',
            payload: {
              url: href,
              referer: location.href,
            },
          })
          .then((response) => {
            if (response?.ok) {
              showNotice('Sent to Lux')
            } else {
              showNotice(response?.error ?? 'Failed to send to Lux', true)
            }
          })
          .catch((error) => {
            showNotice(String(error), true)
          })
      },
      true,
    )
  },
})

function showNotice(message: string, isError = false) {
  const id = 'lux-extension-notice'
  const existing = document.getElementById(id)
  if (existing) {
    existing.remove()
  }

  const notice = document.createElement('div')
  notice.id = id
  notice.textContent = message
  notice.style.position = 'fixed'
  notice.style.top = '16px'
  notice.style.right = '16px'
  notice.style.zIndex = '2147483647'
  notice.style.padding = '10px 14px'
  notice.style.borderRadius = '8px'
  notice.style.fontFamily = 'ui-sans-serif, system-ui, sans-serif'
  notice.style.fontSize = '12px'
  notice.style.background = isError ? '#b91c1c' : '#1d4ed8'
  notice.style.color = '#fff'
  notice.style.boxShadow = '0 8px 20px rgba(0,0,0,0.25)'

  document.documentElement.appendChild(notice)
  setTimeout(() => notice.remove(), 2400)
}
