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
  <BaseDialog :open="open" title="Remove task?" @close="$emit('cancel')">
    <p class="dialog-text">{{ taskTitle }}</p>
    <t-checkbox
      :checked="removeDeleteFile"
      @change="$emit('update:removeDeleteFile', $event)"
    >
      Delete downloaded file from disk
    </t-checkbox>
    <template #actions>
      <t-button variant="outline" @click="$emit('cancel')">Cancel</t-button>
      <t-button theme="danger" @click="$emit('confirm')">Remove</t-button>
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
