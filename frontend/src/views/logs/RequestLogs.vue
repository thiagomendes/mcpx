<template>
  <DashboardLayout>
    <div class="flex items-center justify-between mb-8">
      <div>
        <h1 class="text-2xl font-bold mb-2">Request Logs</h1>
        <p class="text-gray-400">Detailed history of all MCP requests</p>
      </div>
      <div class="flex items-center gap-4">
        <!-- Auto Refresh Toggle -->
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-400">Auto</span>
          <button 
            @click="toggleAutoRefresh"
            class="relative w-12 h-6 rounded-full transition-colors"
            :class="autoRefresh ? 'bg-primary' : 'bg-gray-600'"
          >
            <span 
              class="absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform"
              :class="autoRefresh ? 'left-6' : 'left-0.5'"
            />
          </button>
          <span v-if="autoRefresh" class="text-sm text-gray-400 min-w-[40px]">{{ countdown }}s</span>
        </div>
        <!-- Manual Refresh -->
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
        <div class="flex-1 min-w-[150px]">
          <label class="block text-sm font-medium text-gray-400 mb-2">Server/Gateway</label>
          <input 
            v-model="filterTargetName" 
            type="text" 
            placeholder="e.g. deepwiki"
            class="input w-full"
            @keyup.enter="applyFilters"
          />
        </div>
        <div class="w-40">
          <label class="block text-sm font-medium text-gray-400 mb-2">Method</label>
          <select v-model="filterMethod" class="input w-full">
            <option value="">All</option>
            <option value="initialize">initialize</option>
            <option value="tools/list">tools/list</option>
            <option value="tools/call">tools/call</option>
            <option value="prompts/list">prompts/list</option>
            <option value="resources/list">resources/list</option>
          </select>
        </div>
        <div class="w-32">
          <label class="block text-sm font-medium text-gray-400 mb-2">Status</label>
          <select v-model="filterSuccess" class="input w-full">
            <option value="">All</option>
            <option value="true">Success</option>
            <option value="false">Error</option>
          </select>
        </div>
        <div class="w-40">
          <label class="block text-sm font-medium text-gray-400 mb-2">Time Range</label>
          <select v-model="filterHours" class="input w-full">
            <option :value="1">Last 1 hour</option>
            <option :value="6">Last 6 hours</option>
            <option :value="24">Last 24 hours</option>
            <option :value="168">Last 7 days</option>
            <option :value="720">Last 30 days</option>
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
        <ClipboardDocumentListIcon class="w-8 h-8 text-primary" />
      </div>
      <h3 class="text-lg font-semibold mb-2">No request logs found</h3>
      <p class="text-gray-400">Make some MCP requests to start seeing logs here</p>
    </div>

    <!-- Logs Table -->
    <div v-else class="card overflow-hidden p-0">
      <div class="overflow-x-auto">
      <table class="w-full min-w-[800px]">
        <thead>
          <tr class="border-b border-border bg-background-card">
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Time</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Target</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Method</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Tool</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Latency</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Status</th>
            <th class="text-left text-xs font-semibold uppercase text-gray-400 px-4 py-3">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr 
            v-for="log in store.logs" 
            :key="log.id"
            class="border-b border-border hover:bg-background-hover transition-colors cursor-pointer"
            :class="{ 'bg-red-500/5': !log.success }"
            @click="viewDetail(log.id)"
          >
            <td class="px-4 py-3">
              <div class="flex items-center gap-2 text-sm text-gray-400 font-mono">
                <ClockIcon class="w-4 h-4" />
                {{ formatTime(log.time) }}
              </div>
            </td>
            <td class="px-4 py-3">
              <div class="flex items-center gap-2">
                <span class="text-xs px-2 py-0.5 rounded bg-background-hover text-gray-400">
                  {{ log.target_type }}
                </span>
                <span class="font-medium">{{ log.target_name }}</span>
              </div>
            </td>
            <td class="px-4 py-3">
              <span class="font-mono text-sm">{{ formatMethod(log.method) }}</span>
            </td>
            <td class="px-4 py-3">
              <span v-if="log.tool_name" class="font-mono text-sm text-primary">{{ log.tool_name }}</span>
              <span v-else class="text-gray-500">-</span>
            </td>
            <td class="px-4 py-3">
              <span 
                v-if="log.latency_ms" 
                class="font-mono text-sm"
                :class="log.latency_ms > 2000 ? 'text-yellow-400' : 'text-gray-300'"
              >
                {{ log.latency_ms }}ms
              </span>
              <span v-else class="text-gray-500">-</span>
            </td>
            <td class="px-4 py-3">
              <span v-if="log.success" class="status-badge status-healthy">
                <CheckCircleIcon class="w-3.5 h-3.5" />
                OK
              </span>
              <span v-else class="status-badge status-error">
                <XCircleIcon class="w-3.5 h-3.5" />
                Error
              </span>
            </td>
            <td class="px-4 py-3">
              <button 
                class="text-primary hover:text-primary-light text-sm font-medium"
                @click.stop="viewDetail(log.id)"
              >
                View
              </button>
            </td>
          </tr>
        </tbody>
      </table>
      </div>

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
          <h2 class="text-lg font-semibold">Request Details</h2>
          <button class="btn btn-ghost p-2" @click="closeModal">
            <XMarkIcon class="w-5 h-5" />
          </button>
        </div>
        <div class="p-4 overflow-y-auto flex-1">
          <div class="grid grid-cols-3 gap-4 mb-6">
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Time</label>
              <span class="text-sm">{{ formatTime(store.selectedLog.time) }}</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Target</label>
              <span class="text-sm">{{ store.selectedLog.target_type }}: {{ store.selectedLog.target_name }}</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Method</label>
              <span class="text-sm font-mono">{{ store.selectedLog.method || '-' }}</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Tool</label>
              <span class="text-sm font-mono text-primary">{{ store.selectedLog.tool_name || '-' }}</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Latency</label>
              <span class="text-sm font-mono">{{ store.selectedLog.latency_ms }}ms</span>
            </div>
            <div>
              <label class="block text-xs font-semibold uppercase text-gray-400 mb-1">Status</label>
              <span v-if="store.selectedLog.success" class="status-badge status-healthy">OK</span>
              <span v-else class="status-badge status-error">Error</span>
            </div>
          </div>

          <div v-if="store.selectedLog.error_message" class="mb-4">
            <label class="block text-xs font-semibold uppercase text-gray-400 mb-2">Error Message</label>
            <pre class="bg-red-500/10 border border-red-500/30 rounded-lg p-3 text-sm text-red-400 overflow-x-auto">{{ store.selectedLog.error_message }}</pre>
          </div>

          <div v-if="store.selectedLog.request_body" class="mb-4">
            <label class="block text-xs font-semibold uppercase text-gray-400 mb-2">Request Body</label>
            <pre class="bg-background-hover border border-border rounded-lg p-3 text-sm overflow-x-auto max-h-48">{{ JSON.stringify(store.selectedLog.request_body, null, 2) }}</pre>
          </div>

          <div v-if="store.selectedLog.response_body">
            <label class="block text-xs font-semibold uppercase text-gray-400 mb-2">Response Body</label>
            <pre class="bg-background-hover border border-border rounded-lg p-3 text-sm overflow-x-auto max-h-48">{{ JSON.stringify(store.selectedLog.response_body, null, 2) }}</pre>
          </div>
        </div>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, computed, ref, watch } from 'vue'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useRequestLogsStore } from '@/stores/requestLogs'
import { 
  ClockIcon, 
  CheckCircleIcon, 
  XCircleIcon,
  FunnelIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  XMarkIcon,
  ClipboardDocumentListIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

const store = useRequestLogsStore()

// Filters
const filterTargetName = ref('')
const filterMethod = ref('')
const filterSuccess = ref<string>('')
const filterHours = ref(24)

const showDetailModal = ref(false)

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
    target_name: filterTargetName.value || undefined,
    method: filterMethod.value || undefined,
    success: filterSuccess.value === '' ? undefined : filterSuccess.value === 'true',
    hours: filterHours.value,
    offset: 0,
  })
}

function clearFilters() {
  filterTargetName.value = ''
  filterMethod.value = ''
  filterSuccess.value = ''
  filterHours.value = 24
  applyFilters()
}

async function viewDetail(id: string) {
  await store.fetchLogDetail(id)
  showDetailModal.value = true
}

function closeModal() {
  showDetailModal.value = false
  store.clearSelection()
}

function formatTime(isoString: string): string {
  return new Date(isoString).toLocaleString()
}

function formatMethod(method: string | null): string {
  if (!method) return '-'
  return method
}

// Auto-refresh
const autoRefresh = ref(localStorage.getItem('mcpx_request_logs_autorefresh') === 'true')
const countdown = ref(60)
let refreshTimer: ReturnType<typeof setInterval> | null = null
let countdownTimer: ReturnType<typeof setInterval> | null = null

watch(autoRefresh, (val) => localStorage.setItem('mcpx_request_logs_autorefresh', String(val)))

function toggleAutoRefresh() {
  autoRefresh.value = !autoRefresh.value
  if (autoRefresh.value) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
}

function startAutoRefresh() {
  stopAutoRefresh()
  countdown.value = 60
  countdownTimer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      countdown.value = 60
    }
  }, 1000)
  refreshTimer = setInterval(() => {
    refreshAll()
  }, 60000)
}

function stopAutoRefresh() {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
  countdown.value = 60
}

function refreshAll() {
  applyFilters()
}

onMounted(() => {
  store.fetchLogs()
  if (autoRefresh.value) {
    startAutoRefresh()
  }
})

onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<style scoped>
.status-badge {
  @apply inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium;
}

.status-healthy {
  @apply bg-green-500/20 text-green-400;
}

.status-error {
  @apply bg-red-500/20 text-red-400;
}
</style>
