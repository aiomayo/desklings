<script setup lang="ts">
import type { SettingsPage } from '../framework/types.ts'
import { cn } from '@/lib/utils.ts'

defineProps<{
  pages: SettingsPage[]
  activeId: string
}>()

const emit = defineEmits<{ (e: 'select', id: string): void }>()
</script>

<template>
  <nav class="flex h-full w-56 shrink-0 flex-col gap-1 border-r bg-sidebar p-3">
    <div class="px-2 pb-2 pt-1">
      <p class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        Desklings
      </p>
      <p class="text-[10px] text-muted-foreground/70">Settings</p>
    </div>
    <button
      v-for="page in pages"
      :key="page.id"
      type="button"
      :aria-current="page.id === activeId ? 'page' : undefined"
      :class="cn(
        'flex items-center gap-2 rounded-md px-3 py-2 text-left text-sm font-medium transition-colors',
        page.id === activeId
          ? 'bg-sidebar-accent text-sidebar-accent-foreground'
          : 'text-sidebar-foreground/80 hover:bg-sidebar-accent/50 hover:text-sidebar-foreground',
      )"
      @click="emit('select', page.id)"
    >
      <component :is="page.icon" v-if="page.icon" class="size-4 shrink-0" />
      <span class="truncate">{{ page.title }}</span>
    </button>
  </nav>
</template>
