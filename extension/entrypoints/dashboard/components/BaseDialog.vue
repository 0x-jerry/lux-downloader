<script setup lang="ts">
export interface BaseDialogProps {
  open: boolean
  title?: string
  size?: 'sm' | 'lg'
  mode?: 'normal' | 'modal' | 'modeless' | 'full-screen'
}

withDefaults(defineProps<BaseDialogProps>(), {
  title: '',
  size: 'sm',
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()
</script>

<template>
  <t-dialog
    :visible="open"
    :header="title || false"
    :width="size === 'lg' ? '860px' : '420px'"
    :confirm-btn="null"
    :mode="mode"
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
