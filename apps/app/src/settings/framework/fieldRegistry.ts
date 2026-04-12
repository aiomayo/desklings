import type { Component } from 'vue'
import type { FieldKind } from './types.ts'

import SwitchField from './fields/SwitchField.vue'
import SelectField from './fields/SelectField.vue'
import TextField from './fields/TextField.vue'
import NumberField from './fields/NumberField.vue'
import SliderField from './fields/SliderField.vue'
import RadioCardsField from './fields/RadioCardsField.vue'
import CheckboxCardsField from './fields/CheckboxCardsField.vue'
import InfoField from './fields/InfoField.vue'
import ActionField from './fields/ActionField.vue'

export const fieldRegistry: Record<FieldKind, Component> = {
  'switch': SwitchField,
  'select': SelectField,
  'text': TextField,
  'number': NumberField,
  'slider': SliderField,
  'radio-cards': RadioCardsField,
  'checkbox-cards': CheckboxCardsField,
  'info': InfoField,
  'action': ActionField,
}

export function resolveField(kind: FieldKind): Component {
  const component = fieldRegistry[kind]
  if (!component) {
    throw new Error(`[settings] no component registered for field kind "${kind}"`)
  }
  return component
}
