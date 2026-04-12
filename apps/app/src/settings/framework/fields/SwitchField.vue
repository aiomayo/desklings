<script setup lang="ts">
import type { SwitchFieldDef } from '../types.ts'
import { useFieldBinding } from '../useFieldBinding.ts'
import FieldShell from '../FieldShell.vue'
import { Switch } from '@/components/ui/switch'

const props = defineProps<{ field: SwitchFieldDef }>()
const { value, error, commit, loading, saving } = useFieldBinding(props.field.binding)

function onUpdate(next: boolean) {
  void commit(next)
}
</script>

<template>
  <FieldShell
    :field-id="field.id"
    :label="field.label"
    :description="field.description"
    :error="error"
    layout="inline"
  >
    <Switch
      :id="field.id"
      :model-value="value ?? false"
      :disabled="loading || saving || (field.disabled?.() ?? false)"
      @update:model-value="onUpdate"
    />
  </FieldShell>
</template>
