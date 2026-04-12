<script setup lang="ts">
import { ref, watch } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import type { TextFieldDef } from '../types.ts'
import { useFieldBinding } from '../useFieldBinding.ts'
import FieldShell from '../FieldShell.vue'
import { Input } from '@/components/ui/input'

const props = defineProps<{ field: TextFieldDef }>()
const { value, error, commit, loading, saving } = useFieldBinding(props.field.binding)

const draft = ref<string>('')
watch(value, (next) => {
  if (typeof next === 'string') draft.value = next
}, { immediate: true })

const debouncedCommit = useDebounceFn((next: string) => {
  if (next !== value.value) void commit(next)
}, 300)

function onInput(next: string | number) {
  draft.value = String(next)
  debouncedCommit(draft.value)
}
</script>

<template>
  <FieldShell
    :field-id="field.id"
    :label="field.label"
    :description="field.description"
    :error="error"
  >
    <Input
      :id="field.id"
      :model-value="draft"
      :placeholder="field.placeholder"
      :disabled="loading || saving || (field.disabled?.() ?? false)"
      @update:model-value="onInput"
    />
  </FieldShell>
</template>
