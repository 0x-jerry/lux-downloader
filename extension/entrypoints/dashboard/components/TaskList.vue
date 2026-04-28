<script setup lang="ts">
import { computed } from 'vue'
import type { Task } from '../types'
import {
  canChangeSource,
  canPause,
  canRestart,
  canResume,
  isTorrentTask,
  progressText,
} from '../utils'

defineProps<{
  tasks: Task[]
}>()

const emit = defineEmits<{
  action: [id: string, command: 'pause' | 'resume' | 'restart' | 'remove']
  openTorrentDetails: [task: Task]
  openChangeSource: [task: Task]
}>()

const stateTheme = computed(
  () =>
    (task: Task): 'success' | 'danger' | 'warning' | 'primary' | 'default' => {
      const state = task.state.toLowerCase()
      if (['completed', 'seeding'].includes(state)) return 'success'
      if (['failed'].includes(state)) return 'danger'
      if (['paused'].includes(state)) return 'warning'
      if (['downloading', 'metadata_fetching', 'queued'].includes(state)) return 'primary'
      return 'default'
    },
)
</script>

<template>
  <section class="task-list">
    <t-card v-for="task in tasks" :key="task.id" bordered>
      <t-space direction="vertical" size="8px" style="width: 100%">
        <h3>{{ task.spec.destination_path || task.id }}</h3>
        <p class="source">{{ task.spec.source.value }}</p>
        <p class="meta">
          <t-tag variant="light-outline" :theme="stateTheme(task)">{{ task.state }}</t-tag>
          <span>{{ progressText(task) }}</span>
        </p>

        <t-space break-line size="8px">
          <t-button
            v-if="canPause(task)"
            variant="outline"
            size="small"
            @click="$emit('action', task.id, 'pause')"
          >
            Pause
          </t-button>
          <t-button
            v-else-if="canResume(task)"
            variant="outline"
            size="small"
            @click="$emit('action', task.id, 'resume')"
          >
            Resume
          </t-button>
          <t-button
            v-else-if="canRestart(task)"
            variant="outline"
            size="small"
            @click="$emit('action', task.id, 'restart')"
          >
            Restart
          </t-button>
          <t-button
            v-if="canChangeSource(task)"
            variant="outline"
            size="small"
            @click="emit('openChangeSource', task)"
          >
            Change Source
          </t-button>
          <t-button v-if="isTorrentTask(task)" variant="outline" size="small" @click="$emit('openTorrentDetails', task)">
            Show Torrent Details
          </t-button>
          <t-button theme="danger" variant="outline" size="small" @click="$emit('action', task.id, 'remove')">
            Remove
          </t-button>
        </t-space>
      </t-space>
    </t-card>
    <p v-if="!tasks.length" class="empty">No tasks yet.</p>
  </section>
</template>

<style scoped>
.task-list {
  display: grid;
  gap: 10px;
}

h3 {
  margin: 0;
  font-size: 16px;
}

.source {
  margin: 0;
  color: #475569;
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.meta {
  margin: 0;
  color: #334155;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.empty {
  margin: 0;
  color: #64748b;
}
</style>
