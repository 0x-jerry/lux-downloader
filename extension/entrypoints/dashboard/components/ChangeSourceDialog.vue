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
  <BaseDialog :open="open" title="Change task source" @close="$emit('cancel')">
    <p class="dialog-text">{{ taskTitle }}</p>
    <t-form layout="vertical">
      <t-form-item label="Source">
        <t-input
          :value="value"
          type="text"
          clearable
          placeholder="https://... or magnet:?... or .torrent URL"
          @update:value="$emit('update:value', String($event ?? ''))"
        />
      </t-form-item>
    </t-form>

    <template #actions>
      <t-button variant="outline" @click="$emit('cancel')">Cancel</t-button>
      <t-button theme="primary" @click="$emit('confirm')">Save</t-button>
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
