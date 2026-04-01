import { defineConfig } from 'wxt'

export default defineConfig({
  modules: ['@wxt-dev/module-vue'],
  webExt: {
    chromiumProfile: '.profile/chrome',
    keepProfileChanges: true,
    startUrls: ['chrome://new-tab'],
  },
  manifest: {
    name: 'Lux Downloader',
    description: 'Intercept download links and send them to Lux.',
    permissions: ['storage', 'contextMenus', 'cookies'],
    host_permissions: ['<all_urls>'],
    action: {
      default_title: 'Lux Downloader',
    },
  },
  dev: {
    server: {
      port: 3004,
    },
  },
})
