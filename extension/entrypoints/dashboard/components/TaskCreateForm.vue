<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useInjectTasks } from '../composables/useTasks'
import { filenameFromUrl } from '../../../shared'
import CreateTaskRetryDialog from './CreateTaskRetryDialog.vue'

const { createTask } = useInjectTasks()

const form = reactive({
  url: '',
  referer: '',
  creating: false,
  status: '',
})

interface RetryDialog {
  open: (url: string, referer: string, filename: string, error: string) => void
  setError: (msg: string) => void
}
const retryDialog = ref<RetryDialog | null>(null)

async function handleSubmit() {
  if (!form.url.trim()) {
    form.status = 'Enter a URL first.'
    return
  }

  form.creating = true
  form.status = 'Creating task...'

  const result = await createTask(form.url.trim(), form.referer.trim())

  if (!result.ok) {
    const retryUrl = form.url.trim()
    const retryReferer = form.referer.trim()
    const retryFilename = filenameFromUrl(retryUrl)
    form.url = ''
    form.referer = ''
    form.status = ''
    retryDialog.value!.open(retryUrl, retryReferer, retryFilename, result.error)
  } else {
    form.status = `Task created: ${result.id}`
    form.url = ''
    form.referer = ''
  }

  form.creating = false
}

async function handleRetry(retryUrl: string, retryReferer: string, filename: string, overwrite: boolean) {
  form.creating = true
  form.status = 'Retrying...'

  const result = await createTask(retryUrl, retryReferer, {
    destinationPath: filename,
    overwrite,
  })

  if (!result.ok) {
    form.status = ''
    retryDialog.value!.setError(result.error)
  } else {
    form.status = `Task created: ${result.id}`
  }

  form.creating = false
}
</script>

<template>
  <t-form class="add-task-form" layout="vertical" @submit="handleSubmit">
    <t-form-item label="URL">
      <t-input v-model="form.url" type="text" clearable placeholder="https://example.com/file.zip or magnet:..." />
    </t-form-item>

    <t-form-item label="Referer">
      <t-input v-model="form.referer" type="url" clearable placeholder="https://origin-page.example/" />
    </t-form-item>

    <t-space direction="vertical" size="8px" style="width: 100%">
      <t-button :loading="form.creating" type="submit" theme="primary" block> Add Task </t-button>
      <p class="status">{{ form.status }}</p>
    </t-space>
  </t-form>

  <CreateTaskRetryDialog ref="retryDialog" @retry="handleRetry" />
</template>

<style scoped>
.add-task-form {
  margin-bottom: 10px;
}

.status {
  margin: 0;
  color: #334155;
  min-height: 20px;
}
</style>
