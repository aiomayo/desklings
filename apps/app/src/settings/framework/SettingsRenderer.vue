<script setup lang="ts">
import type { SettingsPage } from './types.ts'
import SectionRenderer from './SectionRenderer.vue'

defineProps<{ page: SettingsPage }>()
</script>

<template>
  <div class="flex flex-col gap-6">
    <header class="flex flex-col gap-1">
      <h1 class="text-2xl font-semibold tracking-tight">{{ page.title }}</h1>
      <p v-if="page.description" class="text-sm text-muted-foreground">
        {{ page.description }}
      </p>
    </header>
    <SectionRenderer
      v-for="section in page.sections"
      :key="section.id"
      :section="section"
    />
  </div>
</template>
