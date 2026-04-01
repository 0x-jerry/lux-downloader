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
    <h1>Lux Downloader</h1>
    <form @submit.prevent="submit">
      <input v-model="url" type="url" placeholder="https://example.com/file.zip" />
      <button :disabled="loading" type="submit">Add Task</button>
    </form>

    <p class="status">{{ status }}</p>

    <section v-if="latest" class="latest">
      <h2>Latest Task</h2>
      <p class="mono">{{ latest.id }}</p>
      <p>
        <strong>{{ latest.state }}</strong>
      </p>
      <p class="clip">{{ latest.spec?.source?.value }}</p>
    </section>

    <a :href="dashboardUrl" target="_blank" rel="noreferrer">Open Dashboard & Settings</a>
  </main>
</template>

<style scoped>
.popup {
  width: 320px;
  min-height: 280px;
  padding: 14px;
  font-family:
    ui-sans-serif,
    system-ui,
    -apple-system,
    sans-serif;
  color: #0f172a;
}

h1 {
  margin: 0 0 10px;
  font-size: 18px;
}

h2 {
  margin: 0 0 8px;
  font-size: 13px;
}

form {
  display: grid;
  gap: 8px;
}

input {
  width: 100%;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  padding: 8px;
}

button {
  border: none;
  border-radius: 8px;
  background: #0ea5e9;
  color: white;
  font-weight: 600;
  padding: 8px;
  cursor: pointer;
}

button:disabled {
  opacity: 0.6;
  cursor: default;
}

.status {
  margin: 10px 0;
  min-height: 20px;
  font-size: 12px;
  color: #334155;
}

.latest {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 10px;
  margin-bottom: 10px;
}

.mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
}

.clip {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

a {
  color: #0369a1;
  text-decoration: none;
  font-size: 12px;
}
</style>
