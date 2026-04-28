<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

type TaskItem = {
  id: string
  state: string
  spec: { source: { value: string } }
}

const url = ref('')
const status = ref('')
const loading = ref(false)
const latest = ref<TaskItem | null>(null)

const dashboardUrl = computed(() => browser.runtime.getURL('/dashboard.html'))

onMounted(async () => {
  await refreshLatest()
})

async function refreshLatest() {
  const response = await browser.runtime.sendMessage({ action: 'list_tasks' })
  if (!response?.ok) {
    status.value = response?.error ?? 'Failed to load tasks'
    return
  }

  const items = (response.data?.items ?? []) as TaskItem[]
  latest.value = items[0] ?? null
}

async function submit() {
  if (!url.value.trim()) {
    status.value = 'Enter a URL first.'
    return
  }

  loading.value = true
  status.value = 'Sending to Lux...'

  try {
    const response = await browser.runtime.sendMessage({
      action: 'manual_add_task',
      payload: {
        url: url.value.trim(),
        referer: '',
      },
    })

    if (!response?.ok) {
      status.value = response?.error ?? 'Failed to create task'
      return
    }

    status.value = `Task created: ${response.data.id}`
    url.value = ''
    await refreshLatest()
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <main class="popup">
    <t-space direction="vertical" size="12px" style="width: 100%">
      <h1>Lux Downloader</h1>

      <form class="form" @submit.prevent="submit">
        <t-input
          v-model="url"
          type="url"
          clearable
          placeholder="https://example.com/file.zip"
        />
        <t-button :loading="loading" theme="primary" type="submit" block>
          Add Task
        </t-button>
      </form>

      <p class="status">{{ status }}</p>

      <t-card v-if="latest" title="Latest Task" bordered>
        <t-space direction="vertical" size="4px" style="width: 100%">
          <p class="mono">{{ latest.id }}</p>
          <t-tag variant="light-outline">{{ latest.state }}</t-tag>
          <p class="clip">{{ latest.spec?.source?.value }}</p>
        </t-space>
      </t-card>

      <t-link :href="dashboardUrl" target="_blank" theme="primary" hover="underline">
        Open Dashboard & Settings
      </t-link>
    </t-space>
  </main>
</template>

<style scoped>
.popup {
  width: 320px;
  min-height: 280px;
  padding: 14px;
  color: #0f172a;
}

h1 {
  margin: 0;
  font-size: 18px;
}

.form {
  display: grid;
  gap: 8px;
}

.status {
  margin: 0;
  min-height: 20px;
  font-size: 12px;
  color: #334155;
}

.mono {
  margin: 0;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
}

.clip {
  margin: 0;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
