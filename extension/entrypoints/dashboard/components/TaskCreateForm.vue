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
    <label>
      URL
      <input
        :value="newTaskUrl"
        type="text"
        placeholder="https://example.com/file.zip or magnet:..."
        @input="$emit('update:newTaskUrl', ($event.target as HTMLInputElement).value)"
      />
    </label>
    <label>
      Referer (optional)
      <input
        :value="newTaskReferer"
        type="url"
        placeholder="https://origin-page.example/"
        @input="$emit('update:newTaskReferer', ($event.target as HTMLInputElement).value)"
      />
    </label>
    <div class="row">
      <button :disabled="creatingTask" type="submit">Add Task</button>
      <p class="status">{{ createTaskStatus }}</p>
    </div>
  </form>
</template>

<style scoped>
.add-task-form {
  display: grid;
  gap: 10px;
  margin-bottom: 10px;
}

label {
  display: grid;
  gap: 6px;
  font-size: 14px;
}

input[type='url'],
input[type='text'] {
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  padding: 10px;
}

.row {
  display: grid;
  gap: 8px;
}

.status {
  margin: 0;
  color: #334155;
  min-height: 20px;
}
</style>
