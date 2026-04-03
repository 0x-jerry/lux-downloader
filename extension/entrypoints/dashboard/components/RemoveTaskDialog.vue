<script setup lang="ts">
import BaseDialog from './BaseDialog.vue'

defineProps<{
  open: boolean
  taskTitle: string
  removeDeleteFile: boolean
}>()

defineEmits<{
  cancel: []
  confirm: []
  'update:removeDeleteFile': [value: boolean]
}>()
</script>

<template>
  <BaseDialog
    :open="open"
    title="Remove task?"
    title-id="remove-task-title"
    @close="$emit('cancel')"
  >
    <p class="dialog-text">{{ taskTitle }}</p>
    <label class="checkbox">
      <input
        :checked="removeDeleteFile"
        type="checkbox"
        @change="$emit('update:removeDeleteFile', ($event.target as HTMLInputElement).checked)"
      />
      Delete downloaded file from disk
    </label>
    <template #actions>
        <button @click="$emit('cancel')">Cancel</button>
        <button class="danger" @click="$emit('confirm')">Remove</button>
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

.checkbox {
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: center;
  gap: 8px;
}

.danger {
  border-color: #fca5a5;
  color: #b91c1c;
}
</style>
