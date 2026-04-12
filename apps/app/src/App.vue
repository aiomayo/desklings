<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';

interface DesklingInfo {
  name: string;
  size: number;
  sprites_dir: string;
  version: number;
}

interface DesklingState {
  sprite: string;
  flip: boolean;
  mode: string;
}

const info = ref<DesklingInfo | null>(null);
const state = ref<DesklingState>({ sprite: '', flip: false, mode: '' });

function joinPath(dir: string, file: string): string {
  const separator = dir.includes('\\') && !dir.includes('/') ? '\\' : '/';
  return dir.endsWith(separator) ? `${dir}${file}` : `${dir}${separator}${file}`;
}

const spriteSrc = computed(() => {
  if (!info.value || !state.value.sprite) return '';
  const absPath = joinPath(info.value.sprites_dir, state.value.sprite);
  const url = convertFileSrc(absPath);
  return info.value.version > 0 ? `${url}?v=${info.value.version}` : url;
});

const sizePx = computed(() => info.value?.size ?? 256);

let unlistenState: UnlistenFn | null = null;
let unlistenReload: UnlistenFn | null = null;

async function refreshDesklingInfo() {
  info.value = await invoke<DesklingInfo | null>('get_deskling_info');
}

onMounted(async () => {
  await refreshDesklingInfo();

  const win = getCurrentWebviewWindow();

  unlistenState = await win.listen<DesklingState>('deskling_state', (event) => {
    state.value = event.payload;
  });

  unlistenReload = await win.listen<string>('deskling_reloaded', async () => {
    await refreshDesklingInfo();
  });
});

onUnmounted(() => {
  unlistenState?.();
  unlistenReload?.();
});
</script>

<template>
  <div
    v-if="info"
    class="deskling"
    :class="{ flipped: state.flip }"
    :style="{ width: `${sizePx}px`, height: `${sizePx}px` }"
  >
    <img
      v-if="spriteSrc"
      :src="spriteSrc"
      :alt="info.name"
      class="sprite"
      :style="{ width: `${sizePx}px`, height: `${sizePx}px` }"
      draggable="false"
    />
  </div>
</template>

<style>
html,
body,
#app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: transparent;
  -webkit-user-select: none;
  user-select: none;
  -webkit-user-drag: none;
  cursor: default;
}
</style>

<style scoped>
.deskling {
  display: flex;
  align-items: center;
  justify-content: center;
  transition: none;
}

.deskling.flipped {
  transform: scaleX(-1);
}

.sprite {
  image-rendering: auto;
  pointer-events: none;
  -webkit-user-drag: none;
}
</style>
