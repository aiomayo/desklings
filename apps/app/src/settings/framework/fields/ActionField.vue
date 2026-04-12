<script setup lang="ts">
import { ref } from 'vue'
import type { ActionFieldDef } from '../types.ts'
import { Button } from '@/components/ui/button'

const props = defineProps<{ field: ActionFieldDef }>()

const running = ref(false)
const error = ref<string | null>(null)

async function onClick() {
  running.value = true
  error.value = null
  try {
    await props.field.run()
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    running.value = false
  }
}
</script>

<template>
  <div class="flex items-start justify-between gap-4">
    <div class="flex flex-col gap-1">
      <span class="text-sm font-medium">{{ field.label }}</span>
      <span v-if="field.description" class="text-xs text-muted-foreground">
        {{ field.description }}
      </span>
      <span v-if="error" class="text-xs text-destructive">{{ error }}</span>
    </div>
    <Button
      :variant="field.variant ?? 'default'"
      :disabled="running || (field.disabled?.() ?? false)"
      class="shrink-0"
      @click="onClick"
    >
      {{ running ? 'Working…' : field.label }}
    </Button>
  </div>
</template>
