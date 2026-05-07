<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useInjectTasks } from '../composables/useTasks'
import { filenameFromUrl } from '../../../shared'
import CreateTaskRetryDialog, { CreateTaskRetryDialogRetryOptions } from './CreateTaskRetryDialog.vue'

const { createTask } = useInjectTasks()

const form = reactive({
  url: '',
  referer: '',
  creating: false,
  status: '',
})

const retryDialogOpen = ref(false)

interface RetryDialogData {
  url: string
  referer: string
  filename: string
  error: string
}

const retryDialogData = reactive<RetryDialogData>({
  url: '',
  referer: '',
  filename: '',
  error: '',
})

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
    retryDialogData.url = retryUrl
    retryDialogData.referer = retryReferer
    retryDialogData.filename = retryFilename
    retryDialogData.error = result.error
    retryDialogOpen.value = true
  } else {
    form.status = `Task created: ${result.id}`
    form.url = ''
    form.referer = ''
  }

  form.creating = false
}

async function retryFn({ url, referer, filename, overwrite }: CreateTaskRetryDialogRetryOptions) {
  const result = await createTask(url, referer, {
    destinationPath: filename,
    overwrite,
  })

  if (result.ok) {
    form.status = `Task created: ${result.id}`
  }

  return result
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

  <CreateTaskRetryDialog
    v-model:open="retryDialogOpen"
    :url="retryDialogData.url"
    :referer="retryDialogData.referer"
    :filename="retryDialogData.filename"
    :error="retryDialogData.error"
    :retry-fn="retryFn"
  />
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
