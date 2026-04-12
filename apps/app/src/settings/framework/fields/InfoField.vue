<script setup lang="ts">
import type { InfoFieldDef } from '../types.ts'
import { useFieldBinding } from '../useFieldBinding.ts'

const props = defineProps<{ field: InfoFieldDef }>()
const { value, loading, error } = useFieldBinding(props.field.binding)
</script>

<template>
  <div class="flex items-center justify-between gap-4">
    <div class="flex flex-col gap-0.5">
      <span class="text-sm font-medium">{{ field.label }}</span>
      <span v-if="field.description" class="text-xs text-muted-foreground">
        {{ field.description }}
      </span>
    </div>
    <span
      class="text-sm tabular-nums"
      :class="error ? 'text-destructive' : 'text-muted-foreground'"
    >
      <template v-if="loading">…</template>
      <template v-else-if="error">error</template>
      <template v-else>{{ value }}</template>
    </span>
  </div>
</template>
