import { invoke } from '@tauri-apps/api/core'
import type { ThemeChoice } from '@/composables/useTheme.ts'

export interface DesklingSummary {
  slug: string
  name: string
  size: number
  preview: string | null
}

export interface AppSettings {
  active_desklings: Record<string, number>
  locale: string | null
  theme: ThemeChoice
}

export const api = {
  listDesklings: () => invoke<DesklingSummary[]>('list_desklings'),
  getActiveDesklings: () =>
    invoke<Record<string, number>>('get_active_desklings'),
  enableDeskling: (slug: string) =>
    invoke<void>('enable_deskling', { slug }),
  disableDeskling: (slug: string) =>
    invoke<void>('disable_deskling', { slug }),
  setDesklingQuantity: (slug: string, quantity: number) =>
    invoke<void>('set_deskling_quantity', { slug, quantity }),
  getSettings: () => invoke<AppSettings>('get_settings'),
  setLocale: (locale: string | null) =>
    invoke<void>('set_locale', { locale }),
  getTheme: () => invoke<ThemeChoice>('get_theme'),
  setTheme: (theme: ThemeChoice) => invoke<void>('set_theme', { theme }),
  getAutostartEnabled: () => invoke<boolean>('get_autostart_enabled'),
  setAutostartEnabled: (enabled: boolean) =>
    invoke<void>('set_autostart_enabled', { enabled }),
  appVersion: () => invoke<string>('app_version'),
  appIdentifier: () => invoke<string>('app_identifier'),
  appWebsite: () => invoke<string>('app_website'),
  appAuthor: () => invoke<string>('app_author'),
}
