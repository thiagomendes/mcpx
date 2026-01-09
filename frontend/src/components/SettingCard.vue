<template>
  <div class="setting-card">
    <div class="setting-header">
      <div class="setting-label">{{ meta?.label || setting.key }}</div>
      <div class="setting-unit" v-if="meta?.unit">{{ meta.unit }}</div>
    </div>
    
    <div class="setting-description" v-if="setting.description">
      {{ setting.description }}
    </div>
    
    <div class="setting-input-row">
      <input
        type="number"
        :value="setting.value"
        :min="meta?.min || 0"
        :max="meta?.max || 9999"
        :disabled="!canEdit || saving"
        @change="handleChange"
        class="setting-input"
      />
      
      <div class="setting-bounds" v-if="meta">
        Min: {{ meta.min }}, Max: {{ meta.max }}
      </div>
      
      <div v-if="saving" class="saving-indicator">
        <div class="spinner-small"></div>
        Saving...
      </div>
    </div>
    
    <div class="setting-warning" v-if="meta?.warning">
      <ExclamationTriangleIcon class="w-4 h-4 text-amber-400 shrink-0" />
      {{ meta.warning }}
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Setting, SettingMeta } from '@/stores/settings'
import { ExclamationTriangleIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{
  setting: Setting
  meta: SettingMeta | undefined
  canEdit: boolean
  saving: boolean
}>()

const emit = defineEmits<{
  (e: 'update', key: string, value: string): void
}>()

function handleChange(event: Event) {
  const input = event.target as HTMLInputElement
  const value = input.value
  
  // Validate bounds
  if (props.meta) {
    const numValue = parseInt(value)
    if (numValue < props.meta.min || numValue > props.meta.max) {
      input.value = props.setting.value // Reset to original
      return
    }
  }
  
  emit('update', props.setting.key, value)
}
</script>

<style scoped>
.setting-card {
  background: var(--bg-tertiary, #242438);
  border: 1px solid var(--border-color, #333);
  border-radius: 10px;
  padding: 1.25rem;
  transition: border-color 0.2s;
}

.setting-card:hover {
  border-color: var(--border-hover, #444);
}

.setting-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.setting-label {
  font-weight: 600;
  color: var(--text-primary, #fff);
}

.setting-unit {
  color: var(--text-secondary, #888);
  font-size: 0.85rem;
}

.setting-description {
  color: var(--text-secondary, #888);
  font-size: 0.875rem;
  margin-bottom: 1rem;
  line-height: 1.5;
}

.setting-input-row {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.setting-input {
  width: 120px;
  padding: 0.625rem 0.875rem;
  background: var(--bg-primary, #0f0f1a);
  border: 1px solid var(--border-color, #333);
  border-radius: 6px;
  color: var(--text-primary, #fff);
  font-size: 1rem;
}

.setting-input:focus {
  outline: none;
  border-color: var(--primary, #6366f1);
}

.setting-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.setting-bounds {
  color: var(--text-tertiary, #666);
  font-size: 0.8rem;
}

.saving-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--primary, #6366f1);
  font-size: 0.875rem;
}

.spinner-small {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-color, #333);
  border-top-color: var(--primary, #6366f1);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.setting-warning {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.75rem;
  padding: 0.5rem 0.75rem;
  background: rgba(245, 158, 11, 0.1);
  border-radius: 6px;
  color: #f59e0b;
  font-size: 0.8rem;
}

.warning-icon {
  font-size: 0.9rem;
}
</style>
