import type { Component } from 'vue'

export type FieldKind =
  | 'switch'
  | 'select'
  | 'text'
  | 'number'
  | 'slider'
  | 'radio-cards'
  | 'checkbox-cards'
  | 'info'
  | 'action'

export interface Binding<T> {
  load: () => Promise<T>
  save?: (next: T) => Promise<void>
  watch?: (onChange: () => void) => Promise<() => void>
}

export interface RadioCardOption {
  value: string
  title: string
  subtitle?: string
  imageSrc?: string
  badges?: string[]
}

export interface SelectOption {
  value: string
  label: string
}

interface FieldBase<T> {
  id: string
  label: string
  description?: string
  binding: Binding<T>
  visible?: () => boolean
  disabled?: () => boolean
}

export interface SwitchFieldDef extends FieldBase<boolean> {
  kind: 'switch'
}

export interface SelectFieldDef extends FieldBase<string> {
  kind: 'select'
  options: SelectOption[]
  placeholder?: string
}

export interface TextFieldDef extends FieldBase<string> {
  kind: 'text'
  placeholder?: string
}

export interface NumberFieldDef extends FieldBase<number> {
  kind: 'number'
  min?: number
  max?: number
  step?: number
}

export interface SliderFieldDef extends FieldBase<number> {
  kind: 'slider'
  min: number
  max: number
  step?: number
  unit?: string
}

export interface RadioCardsFieldDef extends FieldBase<string> {
  kind: 'radio-cards'
  options: () => Promise<RadioCardOption[]>
}

export interface CheckboxCardsQuantityConfig {
  min?: number
  max?: number
  onSetQuantity: (value: string, quantity: number) => Promise<void>
}

export interface CheckboxCardsFieldDef
  extends FieldBase<Record<string, number>> {
  kind: 'checkbox-cards'
  options: () => Promise<RadioCardOption[]>
  onEnable: (value: string) => Promise<void>
  onDisable: (value: string) => Promise<void>
  quantity?: CheckboxCardsQuantityConfig
}

export interface InfoFieldDef extends FieldBase<string> {
  kind: 'info'
}

export interface ActionFieldDef {
  kind: 'action'
  id: string
  label: string
  description?: string
  run: () => Promise<void>
  variant?: 'default' | 'destructive' | 'secondary' | 'outline'
  visible?: () => boolean
  disabled?: () => boolean
}

export type Field =
  | SwitchFieldDef
  | SelectFieldDef
  | TextFieldDef
  | NumberFieldDef
  | SliderFieldDef
  | RadioCardsFieldDef
  | CheckboxCardsFieldDef
  | InfoFieldDef
  | ActionFieldDef

export interface Section {
  id: string
  title: string
  description?: string
  fields: Field[]
}

export interface SettingsPage {
  id: string
  title: string
  description?: string
  icon?: Component
  sections: Section[]
}

export interface SettingsManifest {
  pages: SettingsPage[]
}

export interface FieldState<T> {
  value: T | undefined
  loading: boolean
  saving: boolean
  error: string | null
}
