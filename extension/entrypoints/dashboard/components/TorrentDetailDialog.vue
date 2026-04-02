<script setup lang="ts">
import type { TorrentDetailEntry } from '../types'
import { formatBytes } from '../utils'

defineProps<{
  open: boolean
  taskTitle: string
  detail: TorrentDetailEntry | null
}>()

defineEmits<{
  close: []
  refresh: []
}>()

function toDisplay(value: unknown): string {
  if (value === null || value === undefined) {
    return '-'
  }
  return String(value)
}

function peerConnection(peer: Record<string, unknown>): string {
  const counters = (peer.counters ?? {}) as Record<string, unknown>
  return toDisplay(counters.connections ?? counters.connection_attempts)
}

function peerDownloadedPieces(peer: Record<string, unknown>): string {
  const counters = (peer.counters ?? {}) as Record<string, unknown>
  return toDisplay(counters.downloaded_and_checked_pieces ?? counters.fetched_chunks)
}
</script>

<template>
  <div v-if="open" class="dialog-backdrop" @click.self="$emit('close')">
    <section class="dialog" role="dialog" aria-modal="true" aria-labelledby="torrent-detail-title">
      <div class="header">
        <h3 id="torrent-detail-title">Torrent Details</h3>
        <button @click="$emit('close')">Close</button>
      </div>
      <p class="dialog-text">{{ taskTitle }}</p>

      <template v-if="detail?.loading">
        <p class="status">Loading torrent details...</p>
      </template>

      <template v-else-if="detail?.error">
        <p class="status error">{{ detail.error }}</p>
        <button @click="$emit('refresh')">Retry</button>
      </template>

      <template v-else-if="detail?.data">
        <p>
          <strong>{{ detail.data.name || 'Unknown torrent' }}</strong>
          · {{ detail.data.state || '-' }}
        </p>
        <p class="meta"><span>Info Hash: <code>{{ detail.data.info_hash || '-' }}</code></span></p>
        <p class="meta">
          <span>Peers: {{ detail.data.connected_peer_count ?? 0 }}</span>
          <span>
            Progress:
            {{ formatBytes(detail.data.stats?.progress_bytes ?? 0) }}
            /
            {{ detail.data.stats?.total_bytes ? formatBytes(detail.data.stats?.total_bytes ?? 0) : '?' }}
          </span>
          <span>Uploaded: {{ formatBytes(detail.data.stats?.uploaded_bytes ?? 0) }}</span>
        </p>
        <p v-if="detail.data.output_folder" class="meta">Output: {{ detail.data.output_folder }}</p>

        <div v-if="detail.data.files?.length" class="section">
          <p class="section-title">Files ({{ detail.data.files.length }})</p>
          <ul>
            <li v-for="(file, index) in detail.data.files" :key="`${file.name}-${index}`">
              {{ file.name }} · {{ formatBytes(file.length) }} ·
              {{ file.included === false ? 'excluded' : 'included' }}
            </li>
          </ul>
        </div>

        <div class="section">
          <p class="section-title">Connected Peers ({{ detail.data.connected_peers?.length || 0 }})</p>
          <p v-if="!detail.data.connected_peers?.length" class="meta">No connected peers.</p>
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
                <tr v-for="(peer, index) in detail.data.connected_peers" :key="`${peer.address || 'peer'}-${index}`">
                  <td>{{ toDisplay(peer.address) }}</td>
                  <td>{{ toDisplay(peer.state) }}</td>
                  <td>{{ peerConnection(peer as Record<string, unknown>) }}</td>
                  <td>{{ peerDownloadedPieces(peer as Record<string, unknown>) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: rgb(15 23 42 / 35%);
  display: grid;
  place-items: center;
  padding: 14px;
}

.dialog {
  width: min(860px, 100%);
  max-height: calc(100vh - 28px);
  overflow: auto;
  border: 1px solid #cbd5e1;
  background: #fff;
  border-radius: 10px;
  padding: 14px;
  display: grid;
  gap: 10px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

h3 {
  margin: 0;
}

.dialog-text {
  margin: 0;
  color: #475569;
  font-size: 13px;
  word-break: break-all;
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
