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
  <section class="card settings">
    <h2>Settings</h2>

    <label>
      Lux Base URL
      <input v-model="form.baseUrl" type="url" placeholder="http://127.0.0.1:8080" />
    </label>

    <label>
      Bearer Token
      <input v-model="form.authToken" type="text" />
    </label>

    <label class="checkbox">
      <input v-model="form.interceptEnabled" type="checkbox" />
      Enable automatic link interception
    </label>

    <label class="checkbox">
      <input v-model="form.includeReferer" type="checkbox" />
      Include referer header when intercepting
    </label>

    <label class="checkbox">
      <input v-model="form.includeCookies" type="checkbox" />
      Include cookies for intercepted link domain
    </label>

    <div class="row">
      <button :disabled="savingConfig" @click="$emit('save')">Save & Validate</button>
      <p class="status">{{ configStatus }}</p>
    </div>
  </section>
</template>

<style scoped>
.card {
  border: 1px solid #e2e8f0;
  background: #f8fafc;
  border-radius: 10px;
  padding: 12px;
}

.settings {
  display: grid;
  gap: 10px;
  align-content: start;
}

label {
  display: grid;
  gap: 6px;
  font-size: 14px;
}

input[type='url'],
input[type='text'] {
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  padding: 10px;
}

.checkbox {
  grid-template-columns: auto 1fr;
  align-items: center;
  gap: 8px;
}

.row {
  display: grid;
  gap: 8px;
}

.status {
  margin: 0;
  color: #334155;
  min-height: 20px;
}
</style>
