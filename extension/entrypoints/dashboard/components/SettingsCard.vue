<script setup lang="ts">
import { onMounted, reactive } from 'vue'
import type { LuxConfig } from '../../../shared'
import { useConfig } from '../composables/useConfig'

const { load: loadConfig, save: saveConfig } = useConfig()

const form = reactive<LuxConfig>({
  baseUrl: 'http://127.0.0.1:8080',
  authToken: 'change-me',
  interceptEnabled: true,
  includeCookies: true,
  includeReferer: true,
})

const ui = reactive({
  saving: false,
  status: '',
})

onMounted(async () => {
  try {
    const config = await loadConfig()
    Object.assign(form, config)
  } catch (e) {
    ui.status = e instanceof Error ? e.message : String(e)
  }
})

async function handleSave() {
  ui.saving = true
  ui.status = 'Saving...'
  try {
    const result = await saveConfig({ ...form })
    ui.status = result.ok ? 'Saved and validated successfully.' : result.error
  } finally {
    ui.saving = false
  }
}
</script>

<template>
  <t-card title="Settings" bordered>
    <t-form class="settings" layout="vertical">
      <t-form-item label="Lux Base URL">
        <t-input v-model="form.baseUrl" type="url" placeholder="http://127.0.0.1:8080" clearable />
      </t-form-item>

      <t-form-item label="Bearer Token">
        <t-input v-model="form.authToken" type="text" />
      </t-form-item>

      <t-checkbox v-model:checked="form.interceptEnabled">Enable automatic link interception</t-checkbox>
      <t-checkbox v-model:checked="form.includeReferer">Include referer header when intercepting</t-checkbox>
      <t-checkbox v-model:checked="form.includeCookies">Include cookies for intercepted link domain</t-checkbox>

      <t-space direction="vertical" size="8px" style="width: 100%">
        <t-button :loading="ui.saving" theme="primary" block @click="handleSave">
          Save & Validate
        </t-button>
        <p class="status">{{ ui.status }}</p>
      </t-space>
    </t-form>
  </t-card>
</template>

<style scoped>
.settings {
  display: grid;
  gap: 8px;
}

.status {
  margin: 0;
  color: #334155;
  min-height: 20px;
}
</style>
