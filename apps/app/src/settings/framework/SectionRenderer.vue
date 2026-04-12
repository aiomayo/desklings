<script setup lang="ts">
import type { Section } from './types.ts'
import FieldRenderer from './FieldRenderer.vue'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'

defineProps<{ section: Section }>()
</script>

<template>
  <Card class="border-border/60 shadow-xs">
    <CardHeader>
      <CardTitle class="text-base font-semibold">
        {{ section.title }}
      </CardTitle>
      <CardDescription v-if="section.description">
        {{ section.description }}
      </CardDescription>
    </CardHeader>
    <CardContent class="flex flex-col gap-6">
      <FieldRenderer
        v-for="field in section.fields"
        :key="field.id"
        :field="field"
      />
    </CardContent>
  </Card>
</template>
