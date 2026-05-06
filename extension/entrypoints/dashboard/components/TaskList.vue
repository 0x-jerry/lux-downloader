<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Task } from '../types'
import type { TaskPatchRequest } from '../../../shared'
import {
  canChangeSource,
  canPause,
  canRestart,
  canResume,
  isTorrentTask,
  progressText,
} from '../utils'
import { useInjectTasks } from '../composables/useTasks'
import ChangeSourceDialog from './ChangeSourceDialog.vue'
import RemoveTaskDialog from './RemoveTaskDialog.vue'
import TorrentDetailDialog from './TorrentDetailDialog.vue'

const { state, runAction, patchTask } = useInjectTasks()

interface RemoveDialog { open: (id: string) => void }
interface SourceDialog { open: (task: Task) => void }
interface TorrentDialog { open: (task: Task) => void }
const removeDialog = ref<RemoveDialog | null>(null)
const sourceDialog = ref<SourceDialog | null>(null)
const torrentDialog = ref<TorrentDialog | null>(null)

const stateTheme = computed(
  () =>
    (task: Task): 'success' | 'danger' | 'warning' | 'primary' | 'default' => {
      const s = task.state.toLowerCase()
      if (['completed', 'seeding'].includes(s)) return 'success'
      if (['failed'].includes(s)) return 'danger'
      if (['paused'].includes(s)) return 'warning'
      if (['downloading', 'metadata_fetching', 'queued'].includes(s)) return 'primary'
      return 'default'
    },
)

function handleAction(taskId: string, command: 'pause' | 'resume' | 'restart' | 'remove') {
  if (command === 'remove') {
    removeDialog.value!.open(taskId)
    return
  }
  runAction(taskId, command)
}

async function handleRemoveConfirm(taskId: string, deleteFile: boolean) {
  await runAction(taskId, 'remove', deleteFile)
}

function handleChangeSource(task: Task) {
  sourceDialog.value!.open(task)
}

async function handleSourceSave(taskId: string, value: string) {
  const patch: TaskPatchRequest = {
    source: { kind: 'auto', value },
  }
  return patchTask(taskId, patch)
}

function handleTorrentDetails(task: Task) {
  torrentDialog.value!.open(task)
}
</script>

<template>
  <section class="task-list">
    <t-card v-for="task in state.tasks" :key="task.id" bordered>
      <t-space direction="vertical" size="8px" style="width: 100%">
        <h3>{{ task.spec.destination_path || task.id }}</h3>
        <p class="source">{{ task.spec.source.value }}</p>
        <p class="meta">
          <t-tag variant="light-outline" :theme="stateTheme(task)">{{ task.state }}</t-tag>
          <span>{{ progressText(task) }}</span>
        </p>

        <t-space break-line size="8px">
          <t-button v-if="canPause(task)" variant="outline" size="small" @click="handleAction(task.id, 'pause')">
            Pause
          </t-button>
          <t-button v-else-if="canResume(task)" variant="outline" size="small" @click="handleAction(task.id, 'resume')">
            Resume
          </t-button>
          <t-button v-else-if="canRestart(task)" variant="outline" size="small" @click="handleAction(task.id, 'restart')">
            Restart
          </t-button>
          <t-button v-if="canChangeSource(task)" variant="outline" size="small" @click="handleChangeSource(task)">
            Change Source
          </t-button>
          <t-button v-if="isTorrentTask(task)" variant="outline" size="small" @click="handleTorrentDetails(task)">
            Show Torrent Details
          </t-button>
          <t-button theme="danger" variant="outline" size="small" @click="handleAction(task.id, 'remove')">
            Remove
          </t-button>
        </t-space>
      </t-space>
    </t-card>
    <p v-if="!state.tasks.length" class="empty">No tasks yet.</p>

    <RemoveTaskDialog ref="removeDialog" @confirm="handleRemoveConfirm" />
    <ChangeSourceDialog ref="sourceDialog" @save="handleSourceSave" />
    <TorrentDetailDialog ref="torrentDialog" />
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
