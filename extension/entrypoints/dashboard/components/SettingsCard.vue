<script setup lang="ts">
import type { LuxConfig } from '../../../src/shared'

defineProps<{
  form: LuxConfig
  savingConfig: boolean
  configStatus: string
}>()

defineEmits<{
  save: []
}>()
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
        <t-button :loading="savingConfig" theme="primary" block @click="$emit('save')">
          Save & Validate
        </t-button>
        <p class="status">{{ configStatus }}</p>
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
