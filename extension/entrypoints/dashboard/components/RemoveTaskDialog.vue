<script setup lang="ts">
import { reactive } from 'vue'
import BaseDialog from './BaseDialog.vue'
import { useInjectTasks } from '../composables/useTasks'

const { taskTitle } = useInjectTasks()

const state = reactive({
  open: false,
  taskId: null as string | null,
  deleteFile: false,
})

const emit = defineEmits<{
  confirm: [taskId: string, deleteFile: boolean]
}>()

function show(id: string) {
  state.taskId = id
  state.deleteFile = false
  state.open = true
}

function handleCancel() {
  state.open = false
  state.taskId = null
}

function handleConfirm() {
  if (!state.taskId) return
  emit('confirm', state.taskId, state.deleteFile)
  state.open = false
  state.taskId = null
}

defineExpose({ open: show })
</script>

<template>
  <BaseDialog v-model:open="state.open" title="Remove task?">
    <p class="dialog-text">{{ taskTitle(state.taskId) }}</p>
    <t-checkbox v-model:checked="state.deleteFile">
      Delete downloaded file from disk
    </t-checkbox>
    <template #actions>
      <t-button variant="outline" @click="handleCancel">Cancel</t-button>
      <t-button theme="danger" @click="handleConfirm">Remove</t-button>
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
</style>
