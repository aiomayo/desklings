<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { Check, ImageOff, Minus, Plus } from 'lucide-vue-next'
import type { RadioCardOption, CheckboxCardsFieldDef } from '../types.ts'
import { useFieldBinding } from '../useFieldBinding'
import FieldShell from '../FieldShell.vue'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

const props = defineProps<{ field: CheckboxCardsFieldDef }>()
const { value, error, loading, saving, reload } = useFieldBinding(props.field.binding)

const options = ref<RadioCardOption[]>([])
const optionsLoading = ref(false)
const optionsError = ref<string | null>(null)
const busy = ref(false)

const quantityMin = computed(() => props.field.quantity?.min ?? 0)
const quantityMax = computed(() => props.field.quantity?.max)

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

function quantityFor(option: RadioCardOption): number {
  return value.value?.[option.value] ?? 0
}

function isSelected(option: RadioCardOption): boolean {
  return quantityFor(option) > 0
}

async function onToggle(option: RadioCardOption) {
  if (busy.value) return
  busy.value = true
  try {
    if (isSelected(option)) {
      await props.field.onDisable(option.value)
    } else {
      await props.field.onEnable(option.value)
    }
    await reload()
  } catch (e) {
    console.warn('[checkbox-cards] toggle failed:', e)
  } finally {
    busy.value = false
  }
}

async function onAdjustQuantity(
  event: Event,
  option: RadioCardOption,
  delta: number,
) {
  event.stopPropagation()
  if (busy.value) return
  const quantity = props.field.quantity
  if (!quantity) return

  const current = quantityFor(option)
  let next = current + delta
  if (next < quantityMin.value) next = quantityMin.value
  if (quantityMax.value !== undefined && next > quantityMax.value) {
    next = quantityMax.value
  }
  if (next === current) return

  busy.value = true
  try {
    await quantity.onSetQuantity(option.value, next)
    await reload()
  } catch (e) {
    console.warn('[checkbox-cards] quantity adjust failed:', e)
  } finally {
    busy.value = false
  }
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
      role="group"
      :aria-labelledby="`${field.id}-label`"
      class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-3"
    >
      <button
        v-for="option in options"
        :key="option.value"
        type="button"
        role="checkbox"
        :aria-checked="isSelected(option)"
        :disabled="saving || loading || busy || (field.disabled?.() ?? false)"
        :class="cn(
          'group relative flex flex-col overflow-hidden rounded-lg border bg-card text-left shadow-xs transition-all',
          'hover:border-primary/50 hover:shadow-md',
          'focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50',
          'disabled:pointer-events-none disabled:opacity-60',
          isSelected(option)
            ? 'border-primary ring-2 ring-primary/20'
            : 'border-border',
        )"
        @click="onToggle(option)"
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
            v-if="isSelected(option)"
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

          <div
            v-if="field.quantity && isSelected(option)"
            class="mt-2 flex items-center justify-center gap-2"
            @click.stop
          >
            <button
              type="button"
              :aria-label="`Decrease quantity of ${option.title}`"
              :disabled="busy"
              class="flex size-6 items-center justify-center rounded-md border bg-background text-foreground transition-colors hover:bg-muted disabled:pointer-events-none disabled:opacity-50"
              @click="(e) => onAdjustQuantity(e, option, -1)"
            >
              <Minus class="size-3" />
            </button>
            <span
              class="min-w-[1.5rem] text-center text-sm font-medium tabular-nums"
            >
              {{ quantityFor(option) }}
            </span>
            <button
              type="button"
              :aria-label="`Increase quantity of ${option.title}`"
              :disabled="
                busy ||
                (quantityMax !== undefined && quantityFor(option) >= quantityMax)
              "
              class="flex size-6 items-center justify-center rounded-md border bg-background text-foreground transition-colors hover:bg-muted disabled:pointer-events-none disabled:opacity-50"
              @click="(e) => onAdjustQuantity(e, option, 1)"
            >
              <Plus class="size-3" />
            </button>
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
