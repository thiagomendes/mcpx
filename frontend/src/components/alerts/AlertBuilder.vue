<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="card w-full max-w-lg">
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-lg font-semibold">{{ rule ? 'Edit Alert Rule' : 'Create Alert Rule' }}</h2>
        <button class="btn btn-ghost p-2" @click="$emit('close')">
          <XMarkIcon class="w-5 h-5" />
        </button>
      </div>

      <!-- Step Indicators -->
      <div class="flex items-center justify-center gap-4 mb-8">
        <div 
          v-for="s in 2" :key="s"
          class="flex items-center gap-2"
        >
          <div 
            class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-colors"
            :class="step >= s ? 'bg-primary text-white' : 'bg-gray-700 text-gray-400'"
          >
            {{ s }}
          </div>
          <span v-if="s < 2" class="w-8 h-0.5" :class="step > s ? 'bg-primary' : 'bg-gray-700'"></span>
        </div>
      </div>

      <!-- Step 1: Condition -->
      <div v-if="step === 1" class="space-y-4">
        <h3 class="text-sm font-semibold text-gray-400 uppercase mb-4">Define the Condition</h3>
        
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-400 mb-2">Metric</label>
            <select v-model="form.metric" class="input w-full">
              <option v-for="m in metrics" :key="m.value" :value="m.value">{{ m.label }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-400 mb-2">Scope</label>
            <select v-model="form.scope_type" class="input w-full" @change="form.scope_id = undefined">
              <option value="all">All Targets</option>
              <option value="server">Specific Server</option>
              <option value="gateway">Specific Gateway</option>
            </select>
          </div>
        </div>

        <!-- Server/Gateway Selector (conditional) -->
        <div v-if="form.scope_type === 'server'" class="mt-4">
          <label class="block text-sm font-medium text-gray-400 mb-2">Select Server</label>
          <select v-model="form.scope_id" class="input w-full">
            <option :value="undefined" disabled>Choose a server...</option>
            <option v-for="server in serversStore.servers" :key="server.id" :value="server.id">
              {{ server.name }}
            </option>
          </select>
        </div>

        <div v-if="form.scope_type === 'gateway'" class="mt-4">
          <label class="block text-sm font-medium text-gray-400 mb-2">Select Gateway</label>
          <select v-model="form.scope_id" class="input w-full">
            <option :value="undefined" disabled>Choose a gateway...</option>
            <option v-for="gateway in gatewaysStore.gateways" :key="gateway.id" :value="gateway.id">
              {{ gateway.name }}
            </option>
          </select>
        </div>

        <div class="grid grid-cols-3 gap-4 mt-4">
          <div>
            <label class="block text-sm font-medium text-gray-400 mb-2">Operator</label>
            <select v-model="form.operator" class="input w-full">
              <option value="above">Above</option>
              <option value="below">Below</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-400 mb-2">Threshold</label>
            <input 
              v-model.number="form.threshold" 
              type="number" 
              class="input w-full"
              placeholder="5"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-400 mb-2">Duration</label>
            <select v-model.number="form.duration_minutes" class="input w-full">
              <option :value="1">1 min</option>
              <option :value="5">5 min</option>
              <option :value="10">10 min</option>
              <option :value="15">15 min</option>
              <option :value="30">30 min</option>
            </select>
          </div>
        </div>

        <div class="bg-background-hover rounded-lg p-4 text-center">
          <span class="text-gray-300">Alert when <strong class="text-white">{{ formatMetric(form.metric) }}</strong> is <strong class="text-white">{{ form.operator }}</strong> <strong class="text-primary">{{ form.threshold }}{{ getUnit(form.metric) }}</strong> for <strong class="text-white">{{ form.duration_minutes }} minutes</strong></span>
        </div>
      </div>

      <!-- Step 2: Name & Settings -->
      <div v-if="step === 2" class="space-y-4">
        <h3 class="text-sm font-semibold text-gray-400 uppercase mb-4">Name & Settings</h3>
        
        <div>
          <label class="block text-sm font-medium text-gray-400 mb-2">Alert Name</label>
          <input 
            v-model="form.name" 
            type="text" 
            class="input w-full"
            placeholder="e.g., High error rate on deepwiki"
          />
        </div>

        <div class="space-y-3">
          <label class="flex items-center gap-3 cursor-pointer">
            <input 
              type="checkbox" 
              v-model="form.notify_dashboard"
              class="w-5 h-5 rounded border-gray-600 bg-gray-700 text-primary focus:ring-primary"
            />
            <span>Dashboard notification</span>
          </label>
          <label class="flex items-center gap-3 cursor-not-allowed opacity-50">
            <input 
              type="checkbox" 
              disabled
              class="w-5 h-5 rounded border-gray-600 bg-gray-700"
            />
            <span>Email notification <span class="text-xs text-gray-500">(coming soon)</span></span>
          </label>
        </div>
      </div>

      <!-- Navigation Buttons -->
      <div class="flex justify-between mt-8">
        <button 
          v-if="step > 1"
          class="btn btn-ghost"
          @click="step--"
        >
          Back
        </button>
        <div v-else></div>

        <button 
          v-if="step < 2"
          class="btn btn-primary"
          :disabled="!canProceed"
          @click="step++"
        >
          Next
        </button>
        <button 
          v-else
          class="btn btn-primary"
          :disabled="!canSave"
          @click="save"
        >
          {{ rule ? 'Update' : 'Create' }} Alert
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import type { AlertRule, CreateAlertRule } from '@/stores/alerts'
import { useServersStore } from '@/stores/servers'
import { useGatewaysStore } from '@/stores/gateways'
import { XMarkIcon } from '@heroicons/vue/24/outline'

const serversStore = useServersStore()
const gatewaysStore = useGatewaysStore()

const props = defineProps<{
  rule: AlertRule | null
}>()

const emit = defineEmits<{
  close: []
  save: [data: CreateAlertRule]
}>()

const step = ref(1)

const form = ref({
  name: '',
  alert_type: 'threshold',
  metric: 'error_rate',
  scope_type: 'all',
  scope_id: undefined as string | undefined,
  operator: 'above',
  threshold: 5,
  duration_minutes: 5,
  notify_dashboard: true,
})

const metrics = [
  { value: 'error_rate', label: 'Error Rate (%)' },
  { value: 'avg_latency', label: 'Avg Latency (ms)' },
  { value: 'request_count', label: 'Request Count' },
  { value: 'p95_latency', label: 'P95 Latency (ms)' },
]

const canProceed = computed(() => {
  return form.value.threshold > 0
})

const canSave = computed(() => {
  return form.value.name.trim().length > 0
})

function formatMetric(metric: string): string {
  const m = metrics.find(m => m.value === metric)
  return m?.label.split(' ')[0] || metric
}

function getUnit(metric: string): string {
  const units: Record<string, string> = {
    error_rate: '%',
    avg_latency: 'ms',
    request_count: '',
    p95_latency: 'ms',
  }
  return units[metric] || ''
}

function save() {
  emit('save', {
    name: form.value.name,
    alert_type: 'threshold',
    metric: form.value.metric,
    scope_type: form.value.scope_type,
    scope_id: form.value.scope_id,
    operator: form.value.operator,
    threshold: form.value.threshold,
    duration_minutes: form.value.duration_minutes,
    notify_dashboard: form.value.notify_dashboard,
  })
}

onMounted(() => {
  // Load servers and gateways for selector dropdowns
  serversStore.fetchServers()
  gatewaysStore.fetchGateways()
  
  if (props.rule) {
    form.value = {
      name: props.rule.name,
      alert_type: 'threshold',
      metric: props.rule.metric,
      scope_type: props.rule.scope_type,
      scope_id: props.rule.scope_id,
      operator: props.rule.operator,
      threshold: props.rule.threshold,
      duration_minutes: props.rule.duration_minutes,
      notify_dashboard: props.rule.notify_dashboard,
    }
  }
})
</script>
