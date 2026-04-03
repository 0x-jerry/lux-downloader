<script setup lang="ts">
import BaseDialog from './BaseDialog.vue'

defineProps<{
  open: boolean
  taskTitle: string
  value: string
}>()

defineEmits<{
  cancel: []
  confirm: []
  'update:value': [value: string]
}>()
</script>

<template>
  <BaseDialog
    :open="open"
    title="Change task source"
    title-id="change-source-title"
    @close="$emit('cancel')"
  >
    <p class="dialog-text">{{ taskTitle }}</p>
    <label class="field">
      <span>Source</span>
      <input
        :value="value"
        type="text"
        placeholder="https://... or magnet:?... or .torrent URL"
        @input="$emit('update:value', ($event.target as HTMLInputElement).value)"
      />
    </label>
    <template #actions>
      <button @click="$emit('cancel')">Cancel</button>
      <button @click="$emit('confirm')">Save</button>
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

.field {
  display: grid;
  gap: 6px;
}

.field span {
  font-size: 12px;
  color: #334155;
}

.field input {
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  padding: 8px 10px;
  font-size: 14px;
}
</style>
