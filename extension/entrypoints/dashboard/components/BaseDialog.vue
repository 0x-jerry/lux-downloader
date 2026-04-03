<script setup lang="ts">
withDefaults(
  defineProps<{
    open: boolean
    title?: string
    size?: 'sm' | 'lg'
    titleId?: string
  }>(),
  {
    title: '',
    size: 'sm',
    titleId: '',
  },
)

defineEmits<{
  close: []
}>()
</script>

<template>
  <div v-if="open" class="dialog-backdrop" @click.self="$emit('close')">
    <section
      class="dialog"
      :class="size"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="title ? titleId || undefined : undefined"
    >
      <div v-if="title || $slots.headerActions" class="header">
        <h3 v-if="title" :id="titleId || undefined">{{ title }}</h3>
        <slot name="headerActions" />
      </div>
      <div class="content">
        <slot />
      </div>
      <div v-if="$slots.actions" class="actions">
        <slot name="actions" />
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
  max-height: calc(100vh - 28px);
  overflow: auto;
  border: 1px solid #cbd5e1;
  background: #fff;
  border-radius: 10px;
  padding: 14px;
  display: grid;
  gap: 10px;
}

.dialog.lg {
  width: min(860px, 100%);
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

h3 {
  margin: 0;
}

.content {
  display: grid;
  gap: 10px;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
