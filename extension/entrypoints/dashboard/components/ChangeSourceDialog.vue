<script setup lang="ts">
import { reactive } from 'vue'
import BaseDialog from './BaseDialog.vue'
import type { Task } from '../types'

const state = reactive({
  open: false,
  taskTitle: '',
  taskId: null as string | null,
  value: '',
})

const emit = defineEmits<{
  save: [taskId: string, value: string]
}>()

function show(task: Task) {
  state.taskId = task.id
  state.taskTitle = task.spec.destination_path || task.id
  state.value = task.spec.source.value
  state.open = true
}

function handleCancel() {
  state.open = false
  state.taskId = null
}

function handleSave() {
  if (!state.taskId) return
  emit('save', state.taskId, state.value.trim())
  state.open = false
  state.taskId = null
}

defineExpose({ open: show })
</script>

<template>
  <BaseDialog v-model:open="state.open" title="Change task source" size="lg">
    <p class="dialog-text">{{ state.taskTitle }}</p>
    <t-form layout="vertical">
      <t-form-item label="Source">
        <t-input v-model="state.value" type="text" clearable placeholder="https://... or magnet:?... or .torrent URL" />
      </t-form-item>
    </t-form>

    <template #actions>
      <t-button variant="outline" @click="handleCancel">Cancel</t-button>
      <t-button theme="primary" @click="handleSave">Save</t-button>
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
