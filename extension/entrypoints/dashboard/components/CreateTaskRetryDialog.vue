<script setup lang="ts">
import { reactive } from 'vue'
import BaseDialog from './BaseDialog.vue'

const state = reactive({
  open: false,
  error: '',
  url: '',
  referer: '',
  filename: '',
  overwrite: false,
})

const emit = defineEmits<{
  retry: [url: string, referer: string, filename: string, overwrite: boolean]
}>()

function show(retryUrl: string, retryReferer: string, retryFilename: string, retryError: string) {
  state.url = retryUrl
  state.referer = retryReferer
  state.error = retryError
  state.filename = retryFilename
  state.overwrite = false
  state.open = true
}

function setErrorMessage(msg: string) {
  state.error = msg
  state.open = true
}

function handleCancel() {
  state.open = false
}

function handleRetry() {
  state.open = false
  emit('retry', state.url, state.referer, state.filename, state.overwrite)
}

defineExpose({ open: show, setError: setErrorMessage })
</script>

<template>
  <BaseDialog v-model:open="state.open" title="Task creation failed" size="lg">
    <p class="url-display">{{ state.url }}</p>

    <t-alert theme="error" :message="state.error" />

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

    <template #actions>
      <t-button variant="outline" @click="handleCancel">Cancel</t-button>
      <t-button theme="primary" @click="handleRetry">Retry</t-button>
    </template>
  </BaseDialog>
</template>

<style scoped>
.url-display {
  margin: 0;
  color: #475569;
  font-size: 13px;
  word-break: break-all;
}
</style>
