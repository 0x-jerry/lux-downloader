<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import type { InterceptFailure } from '../../shared'

const retrying = ref(false)

const state = reactive({
  url: '',
  referer: '',
  filename: '',
  error: '',
  overwrite: false,
})

onMounted(async () => {
  await loadFailure()
})

async function loadFailure() {
  const stored = await browser.storage.session.get('pendingInterceptFailure')
  if (!stored.pendingInterceptFailure) {
    dismiss()
    return
  }

  const failure = stored.pendingInterceptFailure as InterceptFailure
  state.url = failure.url
  state.referer = failure.referer
  state.filename = failure.filename
  state.error = failure.error
}

async function cleanup() {
  await browser.runtime.sendMessage({ action: 'dismiss_intercept_failure' })
}

async function dismiss() {
  await cleanup()
  window.close()
}

async function handleRetry() {
  retrying.value = true
  const response = await browser.runtime.sendMessage({
    action: 'manual_add_task',
    payload: {
      url: state.url,
      referer: state.referer,
      destinationPath: state.filename,
      overwrite: state.overwrite,
    },
  })
  retrying.value = false

  if (!response?.ok) {
    state.error = response?.error ?? 'Retry failed'
    return
  }

  await dismiss()
}
</script>

<template>
  <main class="retry-page">
    <h2>Task creation failed</h2>

    <p class="url-display">{{ state.url }}</p>

    <t-alert v-if="state.error" theme="error" :message="state.error" />

    <t-form layout="vertical">
      <t-form-item label="Filename">
        <t-input v-model="state.filename" type="text" clearable placeholder="Enter filename..." />
      </t-form-item>

      <t-form-item>
        <t-checkbox v-model:checked="state.overwrite">
          Overwrite if file already exists
        </t-checkbox>
      </t-form-item>
    </t-form>

    <div class="actions">
      <t-button variant="outline" @click="dismiss">Cancel</t-button>
      <t-button theme="primary" :loading="retrying" @click="handleRetry">Retry</t-button>
    </div>
  </main>
</template>

<style>
html,
body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>

<style scoped>
.retry-page {
  padding: 20px;
  display: grid;
  gap: 12px;
}

h2 {
  margin: 0;
  font-size: 16px;
}

.url-display {
  margin: 0;
  color: #475569;
  font-size: 13px;
  word-break: break-all;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
