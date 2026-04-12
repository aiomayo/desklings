<script setup lang="ts">
import { ref, watch } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import type { NumberFieldDef } from '../types.ts'
import { useFieldBinding } from '../useFieldBinding.ts'
import FieldShell from '../FieldShell.vue'
import { Input } from '@/components/ui/input'

const props = defineProps<{ field: NumberFieldDef }>()
const { value, error, commit, loading, saving } = useFieldBinding(props.field.binding)

const draft = ref<number | undefined>(undefined)
watch(value, (next) => {
  if (typeof next === 'number') draft.value = next
}, { immediate: true })

const debouncedCommit = useDebounceFn((next: number) => {
  if (Number.isFinite(next) && next !== value.value) void commit(next)
}, 300)

function onInput(raw: string | number) {
  const num = typeof raw === 'number' ? raw : Number(raw)
  if (Number.isNaN(num)) return
  draft.value = num
  debouncedCommit(num)
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
      type="number"
      :model-value="draft"
      :min="field.min"
      :max="field.max"
      :step="field.step"
      :disabled="loading || saving || (field.disabled?.() ?? false)"
      class="w-32"
      @update:model-value="onInput"
    />
  </FieldShell>
</template>
