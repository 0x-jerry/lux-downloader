<script setup lang="ts">
import BaseDialog from './BaseDialog.vue'

defineProps<{
  open: boolean
  error: string
  url: string
  filename: string
  overwrite: boolean
}>()

defineEmits<{
  cancel: []
  retry: []
  'update:filename': [value: string]
  'update:overwrite': [value: boolean]
}>()
</script>

<template>
  <BaseDialog :open="open" title="Task creation failed" @close="$emit('cancel')" size="lg">
    <p class="url-display">{{ url }}</p>

    <t-alert theme="error" :message="error" />

    <t-form layout="vertical">
      <t-form-item label="Filename">
        <t-input
          :value="filename"
          type="text"
          clearable
          placeholder="Enter filename..."
          @update:value="$emit('update:filename', String($event ?? ''))"
        />
      </t-form-item>

      <t-form-item>
        <t-checkbox
          :checked="overwrite"
          @update:checked="$emit('update:overwrite', $event ?? false)"
        >
          Overwrite if file already exists
        </t-checkbox>
      </t-form-item>
    </t-form>

    <template #actions>
      <t-button variant="outline" @click="$emit('cancel')">Cancel</t-button>
      <t-button theme="primary" @click="$emit('retry')">Retry</t-button>
    </template>
  </BaseDialog>
</template>

<style scoped>
.url-display {
  margin: 0;
  color: #475569;
  font-size: 13px;
  word-break: break-all;
}
</style>
