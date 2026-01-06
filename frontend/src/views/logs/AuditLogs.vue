<template>
  <DashboardLayout>
    <div class="flex items-center justify-between mb-8">
      <div>
        <h1 class="text-2xl font-bold mb-2">Audit Logs</h1>
        <p class="text-gray-400">Security audit trail of admin actions</p>
      </div>
      <div class="flex items-center gap-4">
        <button 
          @click="refreshAll"
          class="p-2 rounded-lg bg-gray-800 text-gray-400 hover:text-white transition-colors"
          :disabled="store.loading"
          title="Refresh now"
        >
          <ArrowPathIcon class="w-4 h-4" :class="{ 'animate-spin': store.loading }" />
        </button>
      </div>
    </div>

    <!-- Filters -->
    <div class="card mb-6">
      <div class="flex items-end gap-4 flex-wrap">
        <div class="w-48">
          <label class="block text-sm font-medium text-gray-400 mb-2">Action</label>
          <select v-model="filterAction" class="input w-full">
            <option value="">All</option>
            <option value="server.create">Server Create</option>
            <option value="server.update">Server Update</option>
            <option value="server.delete">Server Delete</option>
            <option value="gateway.create">Gateway Create</option>
            <option value="gateway.update">Gateway Update</option>
            <option value="gateway.delete">Gateway Delete</option>
            <option value="user.login">User Login</option>
            <option value="user.logout">User Logout</option>
          </select>
        </div>
        <div class="w-40">
          <label class="block text-sm font-medium text-gray-400 mb-2">Resource Type</label>
          <select v-model="filterResourceType" class="input w-full">
            <option value="">All</option>
            <option value="server">Server</option>
            <option value="gateway">Gateway</option>
            <option value="user">User</option>
            <option value="org">Organization</option>
          </select>
        </div>
        <div class="w-40">
          <label class="block text-sm font-medium text-gray-400 mb-2">Time Range</label>
          <select v-model="filterHours" class="input w-full">
            <option :value="24">Last 24 hours</option>
            <option :value="168">Last 7 days</option>
            <option :value="720">Last 30 days</option>
            <option :value="2160">Last 90 days</option>
          </select>
        </div>
        <div class="flex gap-2">
          <button class="btn btn-primary flex items-center gap-2" @click="applyFilters">
            <FunnelIcon class="w-4 h-4" />
            Filter
          </button>
          <button class="btn btn-ghost" @click="clearFilters">Clear</button>
        </div>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="store.loading" class="text-center py-12">
      <div class="text-gray-400">Loading logs...</div>
    </div>

    <!-- Empty State -->
    <div v-else-if="store.logs.length === 0" class="card text-center py-12">
      <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-primary/20 flex items-center justify-center">
        <ShieldCheckIcon class="w-8 h-8 text-primary" />
      </div>
      <h3 class="text-lg font-semibold mb-2">No audit logs found</h3>
      <p class="text-gray-400">Admin actions will be recorded here</p>
    </div>

    <!-- Logs Table -->
    <div v-else class="card overflow-hidden p-0">
      <table class="w-full">
        <thead>
          <tr class="border-b border-border bg-background-card">
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Time</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">User</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Action</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Resource</th>
          </tr>
        </thead>
        <tbody>
        <tr 
            v-for="log in store.logs" 
            :key="log.id"
            class="border-b border-border hover:bg-background-hover transition-colors cursor-pointer"
            @click="viewDetail(log.id)"
          >
            <td class="px-4 py-3">
              <div class="flex items-center gap-2 text-sm text-gray-400 font-mono">
                <ClockIcon class="w-4 h-4" />
                {{ formatTime(log.time) }}
              </div>
            </td>
            <td class="px-4 py-3">
              <div v-if="log.user_name || log.user_email" class="text-sm">
                <div class="font-medium">{{ log.user_name || '-' }}</div>
                <div class="text-xs text-gray-500">{{ log.user_email }}</div>
              </div>
              <span v-else class="text-gray-500">-</span>
            </td>
            <td class="px-4 py-3">
              <span class="px-2 py-1 rounded text-xs font-medium" :class="getActionClass(log.action)">
                {{ formatAction(log.action) }}
              </span>
            </td>
            <td class="px-4 py-3">
              <div class="flex items-center gap-2">
                <span class="text-xs px-2 py-0.5 rounded bg-background-hover text-gray-400">
                  {{ log.resource_type }}
                </span>
                <span class="font-medium">{{ log.resource_name || '-' }}</span>
              </div>
            </td>
          </tr>
        </tbody>
      </table>

      <!-- Pagination -->
      <div class="flex items-center justify-between px-4 py-3 border-t border-border">
        <span class="text-sm text-gray-400">
          Showing {{ store.logs.length }} of {{ store.total }} logs
        </span>
        <div class="flex items-center gap-2">
          <button 
            class="btn btn-ghost p-2" 
            :disabled="currentPage === 1"
            @click="store.prevPage()"
          >
            <ChevronLeftIcon class="w-5 h-5" />
          </button>
          <span class="text-sm text-gray-400 min-w-[100px] text-center">
            Page {{ currentPage }} of {{ totalPages }}
          </span>
          <button 
            class="btn btn-ghost p-2" 
            :disabled="currentPage >= totalPages"
            @click="store.nextPage()"
          >
            <ChevronRightIcon class="w-5 h-5" />
          </button>
        </div>
      </div>
    </div>

    <!-- Detail Modal -->
    <div v-if="showDetailModal && store.selectedLog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="closeModal">
      <div class="card w-full max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <div class="flex items-center justify-between p-4 border-b border-border">
          <h2 class="text-lg font-semibold">Audit Log Details</h2>
          <button class="btn btn-ghost p-2" @click="closeModal">
            <XMarkIcon class="w-5 h-5" />
          </button>
        </div>
        <div class="p-4 overflow-y-auto flex-1">
          <div class="grid grid-cols-2 gap-4 mb-6">
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Time</label>
              <span class="text-sm">{{ formatTime(store.selectedLog.time) }}</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">User</label>
              <div v-if="store.selectedLog.user_name || store.selectedLog.user_email" class="text-sm">
                <div class="font-medium">{{ store.selectedLog.user_name || '-' }}</div>
                <div class="text-xs text-gray-500">{{ store.selectedLog.user_email }}</div>
              </div>
              <span v-else class="text-sm text-gray-500">-</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Action</label>
              <span class="px-2 py-1 rounded text-xs font-medium" :class="getActionClass(store.selectedLog.action)">
                {{ formatAction(store.selectedLog.action) }}
              </span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Resource Type</label>
              <span class="text-sm">{{ store.selectedLog.resource_type }}</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Resource Name</label>
              <span class="text-sm">{{ store.selectedLog.resource_name || '-' }}</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Resource ID</label>
              <span class="text-sm font-mono">{{ store.selectedLog.resource_id || '-' }}</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">IP Address</label>
              <span class="text-sm font-mono">{{ store.selectedLog.ip_address || '-' }}</span>
            </div>
          </div>
          
          <div v-if="store.selectedLog.details" class="mb-4">
            <label class="block text-xs font-semibold uppercase text-gray-400 mb-2">Details</label>
            <div class="bg-background-card rounded-lg p-4 border border-border">
              <pre class="text-sm text-gray-300 overflow-x-auto whitespace-pre-wrap">{{ JSON.stringify(store.selectedLog.details, null, 2) }}</pre>
            </div>
          </div>
          
          <div v-if="store.selectedLog.user_agent" class="mb-4">
            <label class="block text-xs font-semibold uppercase text-gray-400 mb-2">User Agent</label>
            <span class="text-sm text-gray-400">{{ store.selectedLog.user_agent }}</span>
          </div>
        </div>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useAuditLogsStore } from '@/stores/auditLogs'
import { 
  ClockIcon, 
  FunnelIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ShieldCheckIcon,
  ArrowPathIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'

const store = useAuditLogsStore()

// Filters
const filterAction = ref('')
const filterResourceType = ref('')
const showDetailModal = ref(false)
const filterHours = ref(168)

// Computed
const currentPage = computed(() => {
  const offset = store.currentFilters.offset || 0
  const limit = store.currentFilters.limit || 50
  return Math.floor(offset / limit) + 1
})

const totalPages = computed(() => {
  const limit = store.currentFilters.limit || 50
  return Math.ceil(store.total / limit) || 1
})

// Actions
function applyFilters() {
  store.fetchLogs({
    action: filterAction.value || undefined,
    resource_type: filterResourceType.value || undefined,
    hours: filterHours.value,
    offset: 0,
  })
}

function clearFilters() {
  filterAction.value = ''
  filterResourceType.value = ''
  filterHours.value = 168
  applyFilters()
}

function refreshAll() {
  applyFilters()
}

async function viewDetail(logId: string) {
  await store.fetchLogDetail(logId)
  showDetailModal.value = true
}

function closeModal() {
  showDetailModal.value = false
  store.clearSelection()
}

function formatTime(isoString: string): string {
  return new Date(isoString).toLocaleString()
}

function formatAction(action: string): string {
  return action.replace('.', ' → ')
}

function getActionClass(action: string): string {
  if (action.includes('create')) return 'bg-green-500/20 text-green-400'
  if (action.includes('update')) return 'bg-blue-500/20 text-blue-400'
  if (action.includes('delete')) return 'bg-red-500/20 text-red-400'
  if (action.includes('login')) return 'bg-purple-500/20 text-purple-400'
  if (action.includes('logout')) return 'bg-gray-500/20 text-gray-400'
  return 'bg-gray-500/20 text-gray-400'
}

onMounted(() => {
  store.fetchLogs()
})
</script>
