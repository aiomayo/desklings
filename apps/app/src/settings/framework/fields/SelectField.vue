<script setup lang="ts">
import type { SelectFieldDef } from '../types.ts'
import { useFieldBinding } from '../useFieldBinding.ts'
import FieldShell from '../FieldShell.vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { AcceptableValue } from 'reka-ui'

const props = defineProps<{ field: SelectFieldDef }>()
const { value, error, commit, loading, saving } = useFieldBinding(props.field.binding)

function onUpdate(next: AcceptableValue) {
  if (typeof next === 'string') void commit(next)
}
</script>

<template>
  <FieldShell
    :field-id="field.id"
    :label="field.label"
    :description="field.description"
    :error="error"
  >
    <Select
      :model-value="value ?? ''"
      :disabled="loading || saving || (field.disabled?.() ?? false)"
      @update:model-value="onUpdate"
    >
      <SelectTrigger :id="field.id" class="w-full">
        <SelectValue :placeholder="field.placeholder ?? 'Select…'" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem
          v-for="option in field.options"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </SelectItem>
      </SelectContent>
    </Select>
  </FieldShell>
</template>
