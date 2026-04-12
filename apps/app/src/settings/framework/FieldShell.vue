<script setup lang="ts">
import { Label } from '@/components/ui/label'

defineProps<{
  fieldId: string
  label: string
  description?: string
  error?: string | null
  layout?: 'stacked' | 'inline'
}>()
</script>

<template>
  <div
    :class="[
      'flex gap-4',
      layout === 'inline'
        ? 'items-start justify-between'
        : 'flex-col',
    ]"
  >
    <div class="flex flex-col gap-1">
      <Label :for="fieldId" class="text-sm font-medium">{{ label }}</Label>
      <p v-if="description" class="text-xs text-muted-foreground">
        {{ description }}
      </p>
      <p v-if="error" class="text-xs text-destructive">{{ error }}</p>
    </div>
    <div :class="layout === 'inline' ? 'shrink-0' : 'w-full'">
      <slot />
    </div>
  </div>
</template>
