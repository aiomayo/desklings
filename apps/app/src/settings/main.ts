import { createApp } from 'vue'
import '@/styles.css'
import SettingsApp from './shell/SettingsApp.vue'
import { initTheme } from '@/composables/useTheme.ts'

void initTheme()
createApp(SettingsApp).mount('#settings-root')
