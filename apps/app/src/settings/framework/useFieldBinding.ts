import { onBeforeUnmount, onMounted, ref, type Ref } from 'vue'
import type { Binding } from './types.ts'

export interface UseFieldBinding<T> {
  value: Ref<T | undefined>
  loading: Ref<boolean>
  saving: Ref<boolean>
  error: Ref<string | null>
  commit: (next: T) => Promise<void>
  reload: () => Promise<void>
}

export function useFieldBinding<T>(binding: Binding<T>): UseFieldBinding<T> {
  const value = ref<T | undefined>(undefined) as Ref<T | undefined>
  const loading = ref(false)
  const saving = ref(false)
  const error = ref<string | null>(null)
  let unlisten: (() => void) | null = null

  async function reload() {
    loading.value = true
    error.value = null
    try {
      value.value = await binding.load()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function commit(next: T) {
    if (!binding.save) return
    const previous = value.value
    value.value = next
    saving.value = true
    error.value = null
    try {
      await binding.save(next)
    } catch (e) {
      value.value = previous
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      saving.value = false
    }
  }

  onMounted(async () => {
    await reload()
    if (binding.watch) {
      try {
        unlisten = await binding.watch(() => {
          void reload()
        })
      } catch (e) {
        console.warn('[settings] watch subscription failed:', e)
      }
    }
  })

  onBeforeUnmount(() => {
    unlisten?.()
  })

  return { value, loading, saving, error, commit, reload }
}
