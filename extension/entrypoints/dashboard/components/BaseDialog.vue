<script setup lang="ts">
withDefaults(
  defineProps<{
    open: boolean
    title?: string
    size?: 'sm' | 'lg'
  }>(),
  {
    title: '',
    size: 'sm',
  },
)

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()
</script>

<template>
  <t-dialog
    :visible="open"
    :header="title || false"
    :width="size === 'lg' ? '860px' : '420px'"
    placement="center"
    :confirm-btn="null"
    :cancel-btn="null"
    @close="emit('update:open', false)"
  >
    <div class="content">
      <slot />
    </div>

    <template v-if="$slots.actions" #footer>
      <div class="actions">
        <slot name="actions" />
      </div>
    </template>
  </t-dialog>
</template>

<style scoped>
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
