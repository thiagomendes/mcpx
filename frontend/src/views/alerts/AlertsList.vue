<template>
  <DashboardLayout>
    <div class="flex items-center justify-between mb-8">
      <div>
        <h1 class="text-2xl font-bold mb-2">Alert Rules</h1>
        <p class="text-gray-400">Configure alerts to monitor your MCP servers and gateways</p>
      </div>
      <button class="btn btn-primary flex items-center gap-2 whitespace-nowrap" @click="openCreateModal">
        <PlusIcon class="w-5 h-5" />
        Create Alert
      </button>
    </div>

    <!-- Active Alerts Banner -->
    <div v-if="store.activeAlerts.length > 0" class="card mb-6 border-red-500/30 border bg-red-500/5 p-0">
      <div class="p-4 border-b border-red-500/20">
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
          <h3 class="font-semibold text-red-400">{{ store.activeAlerts.length }} Active Alert{{ store.activeAlerts.length > 1 ? 's' : '' }}</h3>
          <span class="text-xs text-gray-500 ml-2">(auto-resolves when condition is no longer met)</span>
        </div>
      </div>
      <div class="divide-y divide-red-500/10">
        <div 
          v-for="alert in store.activeAlerts" 
          :key="alert.id"
          class="p-4 hover:bg-red-500/5 transition-colors"
        >
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <span class="font-medium text-white">{{ alert.rule_name }}</span>
                <span class="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-300">
                  {{ formatScopeLabel(alert) }}
                </span>
              </div>
              <div class="text-sm text-gray-400">
                <span class="text-white">{{ formatMetric(alert.metric) }}</span>: 
                <span class="text-red-400 font-medium">{{ alert.trigger_value.toFixed(1) }}{{ getUnit(alert.metric) }}</span>
                <span class="text-gray-500"> (threshold: {{ alert.threshold }}{{ getUnit(alert.metric) }})</span>
              </div>
            </div>
            <button 
              v-if="alert.status === 'triggered'"
              class="btn btn-ghost text-sm border border-gray-600 hover:border-gray-500"
              @click="acknowledgeAlert(alert.id)"
            >
              Acknowledge
            </button>
            <span v-else class="text-xs text-gray-500 px-2 py-1 bg-gray-700 rounded">Acknowledged</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="store.loading && store.rules.length === 0" class="text-center py-12">
      <div class="text-gray-400">Loading alert rules...</div>
    </div>

    <!-- Empty State -->
    <div v-else-if="store.rules.length === 0" class="card text-center py-12">
      <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-primary/20 flex items-center justify-center">
        <BellIcon class="w-8 h-8 text-primary" />
      </div>
      <h3 class="text-lg font-semibold mb-2">No alert rules yet</h3>
      <p class="text-gray-400 mb-6">Create your first alert rule to start monitoring</p>
      <button class="btn btn-primary whitespace-nowrap" @click="openCreateModal">
        Create Your First Alert
      </button>
    </div>

    <!-- Rules Table -->
    <div v-else class="card overflow-hidden p-0">
      <table class="w-full">
        <thead>
          <tr class="border-b border-border bg-background-card">
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Name</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Type</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Condition</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Scope</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Status</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr 
            v-for="rule in store.rules" 
            :key="rule.id"
            class="border-b border-border hover:bg-background-hover transition-colors cursor-pointer"
            @click="openEditModal(rule)"
          >
            <td class="px-4 py-3">
              <span class="font-medium">{{ rule.name }}</span>
            </td>
            <td class="px-4 py-3">
              <span class="text-xs px-2 py-1 rounded-full" :class="getTypeBadgeClass(rule.alert_type)">
                {{ formatAlertType(rule.alert_type) }}
              </span>
            </td>
            <td class="px-4 py-3">
              <span class="text-sm">
                {{ formatMetric(rule.metric) }} <span class="text-gray-500">is</span> {{ rule.operator }} <span class="text-primary font-medium">{{ rule.threshold }}{{ getUnit(rule.metric) }}</span> <span class="text-gray-500">for</span> {{ rule.duration_minutes }} min
              </span>
            </td>
            <td class="px-4 py-3">
              <span class="text-sm">{{ formatScope(rule.scope_type) }}</span>
            </td>
            <td class="px-4 py-3">
              <button
                @click.stop="toggleEnabled(rule)"
                class="relative w-10 h-5 rounded-full transition-colors"
                :class="rule.enabled ? 'bg-green-500' : 'bg-gray-600'"
              >
                <span 
                  class="absolute top-0.5 w-4 h-4 bg-white rounded-full transition-transform"
                  :class="rule.enabled ? 'left-5' : 'left-0.5'"
                />
              </button>
            </td>
            <td class="px-4 py-3">
              <div class="flex items-center gap-1">
                <button 
                  class="p-1.5 rounded hover:bg-background-hover text-gray-400 hover:text-white transition-colors"
                  @click.stop="openEditModal(rule)"
                  title="Edit"
                >
                  <PencilIcon class="w-4 h-4" />
                </button>
                <button 
                  class="p-1.5 rounded hover:bg-background-hover text-gray-400 hover:text-red-400 transition-colors"
                  @click.stop="confirmDelete(rule)"
                  title="Delete"
                >
                  <TrashIcon class="w-4 h-4" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Alert Builder Modal -->
    <AlertBuilder
      v-if="showBuilder"
      :rule="editingRule"
      @close="closeBuilder"
      @save="handleSave"
    />

    <!-- Delete Confirmation Modal -->
    <div v-if="deleteConfirm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="card w-full max-w-md">
        <h2 class="text-lg font-semibold mb-4">Delete Alert Rule</h2>
        <p class="text-gray-400 mb-6">
          Are you sure you want to delete "<strong class="text-white">{{ deleteConfirm.name }}</strong>"? 
          This action cannot be undone.
        </p>
        <div class="flex justify-end gap-3">
          <button class="btn btn-ghost" @click="deleteConfirm = null">Cancel</button>
          <button class="btn bg-red-500 hover:bg-red-600 text-white" @click="doDelete">Delete</button>
        </div>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import AlertBuilder from '@/components/alerts/AlertBuilder.vue'
import { useAlertsStore, type AlertRule, type CreateAlertRule, type UpdateAlertRule } from '@/stores/alerts'
import { 
  PlusIcon, 
  BellIcon,
  PencilIcon,
  TrashIcon,
} from '@heroicons/vue/24/outline'

const store = useAlertsStore()

const showBuilder = ref(false)
const editingRule = ref<AlertRule | null>(null)
const deleteConfirm = ref<AlertRule | null>(null)

function openCreateModal() {
  editingRule.value = null
  showBuilder.value = true
}

function openEditModal(rule: AlertRule) {
  editingRule.value = rule
  showBuilder.value = true
}

function closeBuilder() {
  showBuilder.value = false
  editingRule.value = null
}

async function handleSave(data: CreateAlertRule | UpdateAlertRule) {
  if (editingRule.value) {
    await store.updateRule(editingRule.value.id, data as UpdateAlertRule)
  } else {
    await store.createRule(data as CreateAlertRule)
  }
  closeBuilder()
}

async function toggleEnabled(rule: AlertRule) {
  await store.toggleRule(rule.id, !rule.enabled)
}

function confirmDelete(rule: AlertRule) {
  deleteConfirm.value = rule
}

async function doDelete() {
  if (deleteConfirm.value) {
    await store.deleteRule(deleteConfirm.value.id)
    deleteConfirm.value = null
  }
}

async function acknowledgeAlert(id: string) {
  await store.acknowledgeAlert(id)
}

function formatAlertType(type: string): string {
  const types: Record<string, string> = {
    threshold: 'Threshold',
    spike: 'Spike',
    no_data: 'No Data',
  }
  return types[type] || type
}

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

function formatScope(scope: string): string {
  const scopes: Record<string, string> = {
    all: 'All Targets',
    server: 'Server',
    gateway: 'Gateway',
  }
  return scopes[scope] || scope
}

function formatScopeLabel(alert: { scope_type: string; scope_name?: string }): string {
  if (alert.scope_type === 'all') {
    return 'All Targets'
  }
  if (alert.scope_name) {
    return alert.scope_name
  }
  return formatScope(alert.scope_type)
}

function getTypeBadgeClass(type: string): string {
  const classes: Record<string, string> = {
    threshold: 'bg-blue-500/20 text-blue-400',
    spike: 'bg-yellow-500/20 text-yellow-400',
    no_data: 'bg-red-500/20 text-red-400',
  }
  return classes[type] || 'bg-gray-500/20 text-gray-400'
}

onMounted(() => {
  store.fetchRules()
  store.fetchActiveAlerts()
})
</script>
