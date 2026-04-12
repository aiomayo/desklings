import {convertFileSrc} from '@tauri-apps/api/core'
import {listen} from '@tauri-apps/api/event'
import {Info, PawPrint, Settings2} from 'lucide-vue-next'
import type {SettingsManifest} from './framework/types.ts'
import {api} from './api.ts'
import {setTheme, type ThemeChoice} from '@/composables/useTheme.ts'

export const manifest: SettingsManifest = {
  pages: [
    {
      id: 'desklings',
      title: 'Desklings',
      description: 'Manage the desklings installed on your system.',
      icon: PawPrint,
      sections: [
        {
          id: 'active',
          title: 'Active desklings',
          description:
            'Toggle which desklings are shown on your desktop. Each enabled deskling gets its own overlay window. Changes take effect instantly.',
          fields: [
            {
              id: 'active_desklings',
              kind: 'checkbox-cards',
              label: 'Installed desklings',
              options: async () => {
                const desklings = await api.listDesklings()
                return desklings.map((d) => ({
                  value: d.slug,
                  title: d.name,
                  subtitle: d.slug,
                  imageSrc: d.preview ? convertFileSrc(d.preview) : undefined,
                  badges: [`${d.size}px`],
                }))
              },
              onEnable: (slug) => api.setDesklingQuantity(slug, 1),
              onDisable: (slug) => api.setDesklingQuantity(slug, 0),
              quantity: {
                min: 0,
                onSetQuantity: (slug, n) => api.setDesklingQuantity(slug, n),
              },
              binding: {
                load: () => api.getActiveDesklings(),
                watch: async (onChange) => {
                  return await listen('deskling_reloaded', () => onChange())
                },
              },
            },
          ],
        },
      ],
    },
    {
      id: 'general',
      title: 'General',
      description: 'App-wide preferences.',
      icon: Settings2,
      sections: [
        {
          id: 'appearance',
          title: 'Appearance',
          description: 'Choose how Desklings looks on your system.',
          fields: [
            {
              id: 'theme',
              kind: 'select',
              label: 'Theme',
              description: 'Follow the system, or force light or dark mode.',
              options: [
                { value: 'system', label: 'System' },
                { value: 'light', label: 'Light' },
                { value: 'dark', label: 'Dark' },
              ],
              binding: {
                load: () => api.getTheme(),
                save: (value) => setTheme(value as ThemeChoice),
                watch: (onChange) =>
                  listen('theme_changed', () => onChange()),
              },
            },
          ],
        },
        {
          id: 'startup',
          title: 'Startup',
          fields: [
            {
              id: 'autostart',
              kind: 'switch',
              label: 'Launch at login',
              description: 'Start Desklings automatically when you log in.',
              binding: {
                load: () => api.getAutostartEnabled(),
                save: (enabled) => api.setAutostartEnabled(enabled),
              },
            },
          ],
        },
      ],
    },
    {
      id: 'about',
      title: 'About',
      description: 'About Desklings.',
      icon: Info,
      sections: [
        {
          id: 'app',
          title: 'Desklings',
          description: 'Custom desktop pet to torture',
          fields: [
            {
              id: 'version',
              kind: 'info',
              label: 'Version',
              binding: { load: () => api.appVersion() },
            },
            {
              id: 'identifier',
              kind: 'info',
              label: 'Bundle identifier',
              binding: { load: () => api.appIdentifier() },
            },
            {
              id: 'website',
              kind: 'info',
              label: 'Website',
              binding: { load: () => api.appWebsite() },
            },
            {
              id: 'author',
              kind: 'info',
              label: 'Author',
              binding: { load: () => api.appAuthor() },
            },
          ],
        },
      ],
    },
  ],
}
