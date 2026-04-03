<script setup lang="ts">
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
</script>

<template>
  <section class="task-list">
    <article v-for="task in tasks" :key="task.id" class="task">
      <h3>{{ task.spec.destination_path || task.id }}</h3>
      <p class="source">{{ task.spec.source.value }}</p>
      <p>
        <strong>{{ task.state }}</strong> · {{ progressText(task) }}
      </p>

      <div class="actions">
        <button v-if="canPause(task)" @click="$emit('action', task.id, 'pause')">Pause</button>
        <button v-else-if="canResume(task)" @click="$emit('action', task.id, 'resume')">Resume</button>
        <button v-else-if="canRestart(task)" @click="$emit('action', task.id, 'restart')">Restart</button>
        <button v-if="canChangeSource(task)" @click="emit('openChangeSource', task)">Change Source</button>
        <button
          v-if="isTorrentTask(task)"
          @click="$emit('openTorrentDetails', task)"
        >
          Show Torrent Details
        </button>
        <button class="danger" @click="$emit('action', task.id, 'remove')">Remove</button>
      </div>
    </article>
    <p v-if="!tasks.length">No tasks yet.</p>
  </section>
</template>

<style scoped>
.task-list {
  display: grid;
  gap: 10px;
}

.task {
  border: 1px solid #e2e8f0;
  background: #fff;
  border-radius: 10px;
  padding: 12px;
}

h3 {
  margin: 0 0 6px;
  font-size: 17px;
}

.source {
  margin: 0 0 8px;
  color: #475569;
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
  flex-wrap: wrap;
}

.danger {
  border-color: #fca5a5;
  color: #b91c1c;
}
</style>
