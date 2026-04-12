import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export type ThemeChoice = 'system' | 'light' | 'dark'
export type ResolvedTheme = 'light' | 'dark'

const STORAGE_KEY = 'desklings.theme'
const THEME_CHANGED_EVENT = 'theme_changed'

const choice = ref<ThemeChoice>(readStoredChoice())
const systemPrefersDark = ref<boolean>(prefersDarkNow())

export const themeChoice = computed<ThemeChoice>(() => choice.value)
export const resolvedTheme = computed<ResolvedTheme>(() =>
  resolve(choice.value, systemPrefersDark.value),
)

let initialized = false
let mediaQuery: MediaQueryList | null = null
let mediaHandler: ((e: MediaQueryListEvent) => void) | null = null
let unlistenThemeEvent: UnlistenFn | null = null

function prefersDarkNow(): boolean {
  if (typeof window === 'undefined' || !window.matchMedia) return false
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function readStoredChoice(): ThemeChoice {
  if (typeof localStorage === 'undefined') return 'system'
  const raw = localStorage.getItem(STORAGE_KEY)
  return raw === 'light' || raw === 'dark' || raw === 'system' ? raw : 'system'
}

function resolve(c: ThemeChoice, systemDark: boolean): ResolvedTheme {
  if (c === 'system') return systemDark ? 'dark' : 'light'
  return c
}

function apply(c: ThemeChoice, systemDark: boolean): void {
  if (typeof document === 'undefined') return
  const resolved = resolve(c, systemDark)
  const root = document.documentElement
  root.classList.toggle('dark', resolved === 'dark')
  root.style.colorScheme = resolved
}

export async function initTheme(): Promise<void> {
  if (initialized) return
  initialized = true

  apply(choice.value, systemPrefersDark.value)

  if (typeof window !== 'undefined' && window.matchMedia) {
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaHandler = (e) => {
      systemPrefersDark.value = e.matches
      apply(choice.value, systemPrefersDark.value)
    }
    mediaQuery.addEventListener('change', mediaHandler)
  }

  try {
    const canonical = await invoke<ThemeChoice>('get_theme')
    if (canonical !== choice.value) {
      choice.value = canonical
      writeStored(canonical)
      apply(canonical, systemPrefersDark.value)
    }
  } catch {}

  try {
    unlistenThemeEvent = await listen<ThemeChoice>(THEME_CHANGED_EVENT, (event) => {
      const next = event.payload
      if (next !== choice.value) {
        choice.value = next
        writeStored(next)
        apply(next, systemPrefersDark.value)
      }
    })
  } catch {}
}

export function disposeTheme(): void {
  if (mediaQuery && mediaHandler) {
    mediaQuery.removeEventListener('change', mediaHandler)
  }
  mediaQuery = null
  mediaHandler = null
  unlistenThemeEvent?.()
  unlistenThemeEvent = null
  initialized = false
}

export async function setTheme(next: ThemeChoice): Promise<void> {
  choice.value = next
  writeStored(next)
  apply(next, systemPrefersDark.value)
  try {
    await invoke<void>('set_theme', { theme: next })
  } catch (err) {
    console.warn('[theme] failed to persist theme to backend', err)
  }
}

function writeStored(c: ThemeChoice): void {
  if (typeof localStorage === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY, c)
  } catch {}
}
