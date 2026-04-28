<script setup lang="ts">
defineProps<{
  newTaskUrl: string
  newTaskReferer: string
  creatingTask: boolean
  createTaskStatus: string
}>()

defineEmits<{
  submit: []
  'update:newTaskUrl': [value: string]
  'update:newTaskReferer': [value: string]
}>()
</script>

<template>
  <form class="add-task-form" @submit.prevent="$emit('submit')">
    <t-form layout="vertical">
      <t-form-item label="URL">
        <t-input
          :value="newTaskUrl"
          type="text"
          clearable
          placeholder="https://example.com/file.zip or magnet:..."
          @update:value="$emit('update:newTaskUrl', String($event ?? ''))"
        />
      </t-form-item>

      <t-form-item label="Referer">
        <t-input
          :value="newTaskReferer"
          type="url"
          clearable
          placeholder="https://origin-page.example/"
          @update:value="$emit('update:newTaskReferer', String($event ?? ''))"
        />
      </t-form-item>

      <t-space direction="vertical" size="8px" style="width: 100%">
        <t-button :loading="creatingTask" type="submit" theme="primary" block>
          Add Task
        </t-button>
        <p class="status">{{ createTaskStatus }}</p>
      </t-space>
    </t-form>
  </form>
</template>

<style scoped>
.add-task-form {
  margin-bottom: 10px;
}

.status {
  margin: 0;
  color: #334155;
  min-height: 20px;
}
</style>
