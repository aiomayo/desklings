<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { Check, ImageOff } from 'lucide-vue-next'
import type { RadioCardOption, RadioCardsFieldDef } from '../types.ts'
import { useFieldBinding } from '../useFieldBinding.ts'
import FieldShell from '../FieldShell.vue'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils.ts'

const props = defineProps<{ field: RadioCardsFieldDef }>()
const { value, error, commit, loading, saving, reload } = useFieldBinding(props.field.binding)

const options = ref<RadioCardOption[]>([])
const optionsLoading = ref(false)
const optionsError = ref<string | null>(null)

async function loadOptions() {
  optionsLoading.value = true
  optionsError.value = null
  try {
    options.value = await props.field.options()
  } catch (e) {
    optionsError.value = e instanceof Error ? e.message : String(e)
  } finally {
    optionsLoading.value = false
  }
}

onMounted(loadOptions)

watch(value, () => {
  void loadOptions()
})

function onSelect(option: RadioCardOption) {
  if (option.value === value.value) return
  void commit(option.value)
}
</script>

<template>
  <FieldShell
    :field-id="field.id"
    :label="field.label"
    :description="field.description"
    :error="error ?? optionsError"
  >
    <div
      v-if="optionsLoading && options.length === 0"
      class="flex h-40 items-center justify-center rounded-md border border-dashed text-sm text-muted-foreground"
    >
      Loading…
    </div>

    <div
      v-else-if="!optionsLoading && options.length === 0"
      class="flex h-40 flex-col items-center justify-center gap-2 rounded-md border border-dashed text-sm text-muted-foreground"
    >
      <ImageOff class="size-5" />
      No options available
    </div>

    <div
      v-else
      role="radiogroup"
      :aria-labelledby="`${field.id}-label`"
      class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-3"
    >
      <button
        v-for="option in options"
        :key="option.value"
        type="button"
        role="radio"
        :aria-checked="option.value === value"
        :disabled="saving || loading || (field.disabled?.() ?? false)"
        :class="cn(
          'group relative flex flex-col overflow-hidden rounded-lg border bg-card text-left shadow-xs transition-all',
          'hover:border-primary/50 hover:shadow-md',
          'focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50',
          'disabled:pointer-events-none disabled:opacity-60',
          option.value === value
            ? 'border-primary ring-2 ring-primary/20'
            : 'border-border',
        )"
        @click="onSelect(option)"
      >
        <div
          class="relative flex aspect-square items-center justify-center overflow-hidden bg-muted/40"
        >
          <img
            v-if="option.imageSrc"
            :src="option.imageSrc"
            :alt="option.title"
            class="size-full object-contain p-4"
            draggable="false"
          />
          <ImageOff v-else class="size-8 text-muted-foreground/50" />

          <span
            v-if="option.value === value"
            class="absolute right-2 top-2 flex size-6 items-center justify-center rounded-full bg-primary text-primary-foreground shadow"
          >
            <Check class="size-3.5" />
          </span>
        </div>

        <div class="flex flex-col gap-1 border-t bg-background/50 p-3">
          <div class="flex items-center justify-between gap-2">
            <span class="truncate text-sm font-medium">{{ option.title }}</span>
          </div>
          <span v-if="option.subtitle" class="truncate text-xs text-muted-foreground">
            {{ option.subtitle }}
          </span>
          <div v-if="option.badges?.length" class="mt-1 flex flex-wrap gap-1">
            <Badge
              v-for="badge in option.badges"
              :key="badge"
              variant="secondary"
              class="text-[10px]"
            >
              {{ badge }}
            </Badge>
          </div>
        </div>
      </button>
    </div>

    <button
      v-if="optionsError"
      type="button"
      class="mt-2 text-xs text-primary underline"
      @click="loadOptions(); reload()"
    >
      Retry
    </button>
  </FieldShell>
</template>
