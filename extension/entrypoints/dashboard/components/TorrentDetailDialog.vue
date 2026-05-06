<script setup lang="ts">
import { computed, reactive, watch } from 'vue'
import type { ConnectedPeer, Task } from '../types'
import { formatBytes, isTorrentTask } from '../utils'
import { useTorrentDetails } from '../composables/useTorrentDetails'
import { useInjectTasks } from '../composables/useTasks'
import BaseDialog from './BaseDialog.vue'

const { cache, fetch: fetchStats, reconcile } = useTorrentDetails()
const { state: tasksState, taskTitle } = useInjectTasks()

const state = reactive({
  open: false,
  taskId: null as string | null,
})

const activeDetail = computed(() => {
  if (!state.taskId) return null
  return cache[state.taskId] ?? null
})

watch(() => tasksState.tasks, (newTasks) => {
  const activeIds = new Set(newTasks.map((t) => t.id))
  reconcile(activeIds)
  if (state.taskId && !activeIds.has(state.taskId)) {
    state.open = false
    state.taskId = null
  }
})

function show(task: Task) {
  if (!isTorrentTask(task)) return
  state.taskId = task.id
  state.open = true
  fetchStats(task.id)
}

function handleRefresh() {
  if (state.taskId) fetchStats(state.taskId)
}

function toDisplay(value: string | number | undefined | null): string {
  if (value === null || value === undefined) return '-'
  return String(value)
}

function peerConnection(peer: ConnectedPeer): string {
  const counters = peer.counters ?? {}
  return toDisplay(counters.connections ?? counters.connection_attempts)
}

function peerDownloadedPieces(peer: ConnectedPeer): string {
  const counters = peer.counters ?? {}
  return toDisplay(counters.downloaded_and_checked_pieces ?? counters.fetched_chunks)
}

defineExpose({ open: show })
</script>

<template>
  <BaseDialog v-model:open="state.open" title="Torrent Details" size="lg" @update:open="(val) => { if (!val) state.taskId = null }">
    <p class="dialog-text">{{ taskTitle(state.taskId) }}</p>

    <template v-if="activeDetail?.loading">
      <t-loading :loading="true" text="Loading torrent details..." />
    </template>

    <template v-else-if="activeDetail?.error">
      <p class="status error">{{ activeDetail.error }}</p>
      <t-button variant="outline" @click="handleRefresh">Retry</t-button>
    </template>

    <template v-else-if="activeDetail?.data">
      <p class="title-line">
        <strong>{{ activeDetail.data.name || 'Unknown torrent' }}</strong>
        <t-tag variant="light-outline">{{ activeDetail.data.state || '-' }}</t-tag>
      </p>
      <p class="meta"><span>Info Hash: <code>{{ activeDetail.data.info_hash || '-' }}</code></span></p>
      <p class="meta">
        <span>Peers: {{ activeDetail.data.connected_peer_count ?? 0 }}</span>
        <span>
          Progress:
          {{ formatBytes(activeDetail.data.stats?.progress_bytes ?? 0) }}
          /
          {{ activeDetail.data.stats?.total_bytes ? formatBytes(activeDetail.data.stats?.total_bytes ?? 0) : '?' }}
        </span>
        <span>Uploaded: {{ formatBytes(activeDetail.data.stats?.uploaded_bytes ?? 0) }}</span>
      </p>
      <p v-if="activeDetail.data.output_folder" class="meta">Output: {{ activeDetail.data.output_folder }}</p>

      <div v-if="activeDetail.data.files?.length" class="section">
        <p class="section-title">Files ({{ activeDetail.data.files.length }})</p>
        <ul>
          <li v-for="(file, index) in activeDetail.data.files" :key="`${file.name}-${index}`">
            {{ file.name }} · {{ formatBytes(file.length) }} ·
            {{ file.included === false ? 'excluded' : 'included' }}
          </li>
        </ul>
      </div>

      <div class="section">
        <p class="section-title">Connected Peers ({{ activeDetail.data.connected_peers?.length || 0 }})</p>
        <p v-if="!activeDetail.data.connected_peers?.length" class="meta">No connected peers.</p>
        <div v-else class="peer-table-wrap">
          <table class="peer-table">
            <thead>
              <tr>
                <th>IP</th>
                <th>State</th>
                <th>Connection</th>
                <th>Download Pieces</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(peer, index) in activeDetail.data.connected_peers" :key="`${peer.address || 'peer'}-${index}`">
                <td>{{ toDisplay(peer.address) }}</td>
                <td>{{ toDisplay(peer.state) }}</td>
                <td>{{ peerConnection(peer) }}</td>
                <td>{{ peerDownloadedPieces(peer) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>

    <template #actions>
      <t-button variant="outline" @click="state.open = false">Close</t-button>
      <t-button v-if="activeDetail?.error || activeDetail?.data" variant="outline" @click="handleRefresh">Refresh</t-button>
    </template>
  </BaseDialog>
</template>

<style scoped>
.dialog-text {
  margin: 0;
  color: #475569;
  font-size: 13px;
  word-break: break-all;
}

.title-line {
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.status {
  margin: 0;
  color: #334155;
}

.error {
  color: #b91c1c;
}

.meta {
  margin: 0;
  color: #334155;
  font-size: 13px;
  display: grid;
  gap: 2px;
}

.section {
  border-top: 1px solid #e2e8f0;
  padding-top: 10px;
  display: grid;
  gap: 8px;
}

.section-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
}

ul {
  margin: 0;
  padding-left: 18px;
}

.peer-table-wrap {
  overflow: auto;
}

.peer-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.peer-table th,
.peer-table td {
  border: 1px solid #e2e8f0;
  padding: 8px;
  text-align: left;
  white-space: nowrap;
}

.peer-table th {
  background: #f8fafc;
}

code {
  font-size: 12px;
}
</style>
