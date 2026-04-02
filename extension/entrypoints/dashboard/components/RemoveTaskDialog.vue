<script setup lang="ts">
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
  <div v-if="open" class="dialog-backdrop" @click.self="$emit('cancel')">
    <section class="dialog" role="dialog" aria-modal="true" aria-labelledby="remove-task-title">
      <h3 id="remove-task-title">Remove task?</h3>
      <p class="dialog-text">{{ taskTitle }}</p>
      <label class="checkbox">
        <input
          :checked="removeDeleteFile"
          type="checkbox"
          @change="$emit('update:removeDeleteFile', ($event.target as HTMLInputElement).checked)"
        />
        Delete downloaded file from disk
      </label>
      <div class="dialog-actions">
        <button @click="$emit('cancel')">Cancel</button>
        <button class="danger" @click="$emit('confirm')">Remove</button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: rgb(15 23 42 / 35%);
  display: grid;
  place-items: center;
  padding: 14px;
}

.dialog {
  width: min(420px, 100%);
  border: 1px solid #cbd5e1;
  background: #fff;
  border-radius: 10px;
  padding: 14px;
  display: grid;
  gap: 10px;
}

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

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.danger {
  border-color: #fca5a5;
  color: #b91c1c;
}
</style>
