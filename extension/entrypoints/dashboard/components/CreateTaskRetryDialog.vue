<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import BaseDialog, { BaseDialogProps } from './BaseDialog.vue'

export interface CreateTaskRetryDialogRetryOptions {
  url: string
  referer: string
  filename: string
  overwrite: boolean
}

export interface CreateTaskRetryDialogProps {
  url: string
  referer: string
  filename: string
  error: string
  mode?: BaseDialogProps['mode']
  retryFn: (options: CreateTaskRetryDialogRetryOptions) => Promise<{ ok: boolean; error?: string }>
}

const props = defineProps<CreateTaskRetryDialogProps>()
const open = defineModel<boolean>('open', { required: true })

const state = reactive({
  filename: props.filename,
  error: props.error,
  overwrite: false,
})

watch(open, (isOpen) => {
  if (isOpen) {
    state.filename = props.filename
    state.error = props.error
    state.overwrite = false
  }
})

const retrying = ref(false)

function handleCancel() {
  open.value = false
}

async function handleRetry() {
  retrying.value = true
  const result = await props.retryFn({ url: props.url, referer: props.referer, filename: state.filename, overwrite: state.overwrite })
  retrying.value = false

  if (!result.ok) {
    state.error = result.error ?? 'Retry failed'
    return
  }

  open.value = false
}
</script>

<template>
  <BaseDialog v-model:open="open" title="Task creation failed" size="lg" :mode="mode">
    <p class="url-display">{{ props.url }}</p>

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
      <t-button theme="primary" :loading="retrying" @click="handleRetry">Retry</t-button>
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
