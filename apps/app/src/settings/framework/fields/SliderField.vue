<script setup lang="ts">
import { computed } from 'vue'
import type { SliderFieldDef } from '../types.ts'
import { useFieldBinding } from '../useFieldBinding.ts'
import FieldShell from '../FieldShell.vue'
import { Slider } from '@/components/ui/slider'

const props = defineProps<{ field: SliderFieldDef }>()
const { value, error, commit, loading, saving } = useFieldBinding(props.field.binding)

const modelValue = computed<number[]>(() => [
  typeof value.value === 'number' ? value.value : props.field.min,
])

function onUpdate(next: number[] | undefined) {
  if (next && next.length > 0) void commit(next[0])
}
</script>

<template>
  <FieldShell
    :field-id="field.id"
    :label="field.label"
    :description="field.description"
    :error="error"
  >
    <div class="flex items-center gap-4">
      <Slider
        :id="field.id"
        :model-value="modelValue"
        :min="field.min"
        :max="field.max"
        :step="field.step ?? 1"
        :disabled="loading || saving || (field.disabled?.() ?? false)"
        class="flex-1"
        @update:model-value="onUpdate"
      />
      <span class="w-16 text-right text-sm tabular-nums text-muted-foreground">
        {{ modelValue[0] }}{{ field.unit ?? '' }}
      </span>
    </div>
  </FieldShell>
</template>
