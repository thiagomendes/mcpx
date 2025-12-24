<template>
  <div v-if="store.activeAlerts.length > 0" class="card p-0 mb-6 border-red-500/30 border">
    <div class="p-4 bg-red-500/10 border-b border-red-500/20">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <ExclamationTriangleIcon class="w-5 h-5 text-red-400" />
          <h3 class="font-semibold text-red-400">{{ store.activeAlerts.length }} Active Alert{{ store.activeAlerts.length > 1 ? 's' : '' }}</h3>
        </div>
        <router-link to="/alerts" class="text-sm text-primary hover:text-primary-light">
          View All
        </router-link>
      </div>
    </div>
    <div class="divide-y divide-border">
      <div 
        v-for="alert in store.activeAlerts.slice(0, 3)" 
        :key="alert.id"
        class="p-4 flex items-center justify-between hover:bg-background-hover transition-colors"
      >
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
          <div>
            <span class="font-medium">{{ alert.rule_name }}</span>
            <span class="text-gray-400 text-sm ml-2">
              {{ formatMetric(alert.metric) }}: {{ alert.trigger_value.toFixed(1) }}{{ getUnit(alert.metric) }}
            </span>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <span class="text-xs text-gray-500">{{ formatTime(alert.triggered_at) }}</span>
          <button 
            v-if="alert.status === 'triggered'"
            class="btn btn-ghost text-xs px-2 py-1"
            @click="acknowledgeAlert(alert.id)"
          >
            Acknowledge
          </button>
          <span v-else class="text-xs text-gray-500">Acknowledged</span>
        </div>
      </div>
    </div>
    <div v-if="store.activeAlerts.length > 3" class="p-3 text-center border-t border-border">
      <router-link to="/alerts" class="text-sm text-gray-400 hover:text-white">
        +{{ store.activeAlerts.length - 3 }} more alerts
      </router-link>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
import { useAlertsStore } from '@/stores/alerts'

const store = useAlertsStore()

function formatMetric(metric: string): string {
  const metrics: Record<string, string> = {
    error_rate: 'Error Rate',
    avg_latency: 'Avg Latency',
    request_count: 'Requests',
    p95_latency: 'P95 Latency',
  }
  return metrics[metric] || metric
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

function formatTime(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  
  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24) return `${diffHours}h ago`
  return date.toLocaleDateString()
}

async function acknowledgeAlert(id: string) {
  await store.acknowledgeAlert(id)
}

onMounted(() => {
  store.fetchActiveAlerts()
})
</script>
