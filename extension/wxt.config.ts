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
    key: 'AAAAB3NzaC1yc2EAAAADAQABAAABAQDJYhTeLbjBvBb7hpi7pnoGjwMfMUrIv22ANHXxaJAzD1/42/5bWgoswe7vLt9eVphFqb49T0R6s+Xbco9JlUmxioXtAtBZ007ZVcdvXULj0RolbMEu38yFo5nxANx+5LbAlUrn4rXyoZH1Qm168EKUQjN/XjjQrEQSBD0E2hBse7DcDRR9Cs3KSVkE9p7Ye+bIw9c4tiXt+DkqBunqy6XTxgW6Dw84+n+tiW5CS6GS5aCqRRbWtW0lzBvFxCwNu2VRMiDXHnkm5Oj4kEHcZlsxnRG5Qe1kFLQgh/eIq0kYTl33OTLbibL4XOwS/KH2daFXvmV/TgIqL1Dzf0AclNat',
    update_url:
      'https://raw.githubusercontent.com/0x-jerry/lux-downloader/main/extension/updates.xml',
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
