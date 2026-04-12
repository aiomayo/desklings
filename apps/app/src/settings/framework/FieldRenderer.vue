<script setup lang="ts">
import { computed } from 'vue'
import type { Field } from './types.ts'
import { resolveField } from './fieldRegistry.ts'

const props = defineProps<{ field: Field }>()

const component = computed(() => resolveField(props.field.kind))
const isVisible = computed(() => (props.field.visible ? props.field.visible() : true))
</script>

<template>
  <component
    :is="component"
    v-if="isVisible"
    :field="field"
  />
</template>
