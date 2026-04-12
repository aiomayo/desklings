<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import SettingsSidebar from './SettingsSidebar.vue'
import SettingsRenderer from '../framework/SettingsRenderer.vue'
import { ScrollArea } from '@/components/ui/scroll-area'
import { manifest } from '../pages.ts'

const STORAGE_KEY = 'desklings.settings.active-page'

function initialPageId(): string {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored && manifest.pages.some((p) => p.id === stored)) return stored
  return manifest.pages[0]?.id ?? ''
}

const activeId = ref(initialPageId())
watch(activeId, (id) => {
  if (id) localStorage.setItem(STORAGE_KEY, id)
})

const activePage = computed(
  () => manifest.pages.find((p) => p.id === activeId.value) ?? manifest.pages[0],
)
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
    <SettingsSidebar
      :pages="manifest.pages"
      :active-id="activeId"
      @select="activeId = $event"
    />
    <ScrollArea class="h-full flex-1">
      <main class="mx-auto max-w-3xl p-8">
        <SettingsRenderer v-if="activePage" :key="activePage.id" :page="activePage" />
      </main>
    </ScrollArea>
  </div>
</template>
