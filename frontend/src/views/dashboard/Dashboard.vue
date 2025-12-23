<template>
  <DashboardLayout>
    <!-- Header with Global Controls -->
    <div class="flex items-center justify-between mb-8 flex-wrap gap-4">
      <div>
        <h1 class="text-2xl font-bold mb-1">Dashboard</h1>
        <p class="text-gray-400">Welcome back, {{ authStore.user?.name || 'User' }}</p>
      </div>
      <div class="flex items-center gap-3 flex-wrap">
        <!-- Server Filter -->
        <select 
          v-model="selectedServer"
          @change="refreshAll"
          class="bg-gray-800 text-sm text-gray-300 rounded-lg px-3 py-2 focus:outline-none cursor-pointer"
        >
          <option value="">All Servers</option>
          <option v-for="server in serversStore.servers" :key="server.id" :value="server.name">
            {{ server.name }}
          </option>
        </select>

        <!-- Time Range Selector -->
        <div class="flex gap-1 bg-gray-800 rounded-lg p-1">
          <button 
            v-for="range in timeRanges" 
            :key="range.hours"
            @click="setTimeRange(range.hours)"
            class="px-3 py-1.5 text-sm rounded-md transition-colors"
            :class="selectedHours === range.hours ? 'bg-primary text-white' : 'text-gray-400 hover:text-white'"
          >
            {{ range.label }}
          </button>
        </div>
        
        <!-- Auto Refresh Toggle Switch -->
        <div class="flex items-center gap-2 bg-gray-800 rounded-lg px-3 py-1.5">
          <span class="text-sm text-gray-400">Auto</span>
          <button
            @click="toggleAutoRefresh"
            class="relative w-11 h-6 rounded-full transition-colors focus:outline-none"
            :class="autoRefresh ? 'bg-green-600' : 'bg-gray-600'"
          >
            <span
              class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform"
              :class="autoRefresh ? 'translate-x-5' : 'translate-x-0'"
            />
          </button>
          <span v-if="autoRefresh" class="text-sm text-green-400 font-mono w-6 text-right">{{ countdown }}s</span>
        </div>
        
        <!-- Manual Refresh -->
        <button 
          @click="refreshAll"
          class="p-2 rounded-lg bg-gray-800 text-gray-400 hover:text-white transition-colors"
          :disabled="loading"
          title="Refresh now"
        >
          <ArrowPathIcon class="w-4 h-4" :class="{ 'animate-spin': loading }" />
        </button>
      </div>
    </div>

    <!-- Quick Stats Row -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
      <div class="card text-center">
        <div class="text-3xl font-bold text-green-400">{{ totalRequests }}</div>
        <div class="text-sm text-gray-400">Requests</div>
      </div>
      <div class="card text-center">
        <div class="text-3xl font-bold" :class="errorRateClass">{{ errorRateFormatted }}</div>
        <div class="text-sm text-gray-400">Error Rate</div>
      </div>

      <!-- Latency Card with Toggle -->
      <div class="card text-center">
        <div class="text-3xl font-bold text-blue-400">{{ currentLatencyFormatted }}</div>
        <div class="text-sm text-gray-400 mb-2">{{ latencyModeLabel }} Latency</div>
        <div class="flex justify-center gap-1">
          <button 
            v-for="mode in latencyModes" :key="mode.key"
            @click="selectedLatencyMode = mode.key"
            class="px-2 py-0.5 text-xs rounded transition-colors"
            :class="selectedLatencyMode === mode.key ? 'bg-blue-500 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'"
          >
            {{ mode.label }}
          </button>
        </div>
      </div>

      <!-- Throughput Card with Toggle -->
      <div class="card text-center">
        <div class="text-3xl font-bold text-purple-400">{{ currentThroughputFormatted }}</div>
        <div class="text-sm text-gray-400 mb-2">{{ throughputModeLabel }} req/min</div>
        <div class="flex justify-center gap-1">
          <button 
            v-for="mode in throughputModes" :key="mode.key"
            @click="selectedThroughputMode = mode.key"
            class="px-2 py-0.5 text-xs rounded transition-colors"
            :class="selectedThroughputMode === mode.key ? 'bg-purple-500 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'"
          >
            {{ mode.label }}
          </button>
        </div>
      </div>
    </div>

    <!-- Charts Grid Row 1 -->
    <div class="grid md:grid-cols-2 gap-6 mb-6">
      <!-- Request Throughput Chart -->
      <div class="card">
        <div class="flex items-center gap-2 mb-4">
          <ChartBarIcon class="w-5 h-5 text-primary" />
          <h3 class="font-semibold">Server Requests <span class="text-gray-500 text-sm font-normal">(requests)</span></h3>
        </div>
        <div class="h-48">
          <Line v-if="throughputData.labels.length > 0" :data="throughputData" :options="lineOptions" />
          <div v-else class="flex items-center justify-center h-full text-gray-500">
            <span v-if="loading" class="animate-pulse">Loading...</span>
            <span v-else>No data</span>
          </div>
        </div>
      </div>

      <!-- Latency Chart -->
      <div class="card">
        <div class="flex items-center gap-2 mb-4">
          <ClockIcon class="w-5 h-5 text-blue-400" />
          <h3 class="font-semibold">Server Latency <span class="text-gray-500 text-sm font-normal">(ms)</span></h3>
        </div>
        <div class="h-48">
          <Line v-if="latencyData.labels.length > 0" :data="latencyData" :options="latencyOptions" />
          <div v-else class="flex items-center justify-center h-full text-gray-500">
            <span v-if="loading" class="animate-pulse">Loading...</span>
            <span v-else>No data</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Tool Calls Over Time (line chart with one line per tool) -->
    <div class="card mb-6">
      <div class="flex items-center gap-2 mb-4">
        <WrenchScrewdriverIcon class="w-5 h-5 text-orange-400" />
        <h3 class="font-semibold">Tool Calls <span class="text-gray-500 text-sm font-normal">(calls)</span></h3>
      </div>
      <div class="h-64">
        <Line v-if="toolCallsTimeData.labels.length > 0" :data="toolCallsTimeData" :options="toolCallsTimeOptions" />
        <div v-else class="flex items-center justify-center h-full text-gray-500">
          <span v-if="loading" class="animate-pulse">Loading...</span>
          <span v-else>No tool call data</span>
        </div>
      </div>
    </div>

    <!-- Tool Latency Over Time (line chart with one line per tool) -->
    <div class="card mb-6">
      <div class="flex items-center gap-2 mb-4">
        <ClockIcon class="w-5 h-5 text-cyan-400" />
        <h3 class="font-semibold">Tool Latency <span class="text-gray-500 text-sm font-normal">(ms)</span></h3>
      </div>
      <div class="h-64">
        <Line v-if="toolLatencyTimeData.labels.length > 0" :data="toolLatencyTimeData" :options="toolLatencyTimeOptions" />
        <div v-else class="flex items-center justify-center h-full text-gray-500">
          <span v-if="loading" class="animate-pulse">Loading...</span>
          <span v-else>No latency data</span>
        </div>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, computed, ref, watch } from 'vue'
import { Line } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip,
  Legend,
  Filler
} from 'chart.js'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useAuthStore } from '@/stores/auth'
import { useServersStore } from '@/stores/servers'
import { useMetricsStore } from '@/stores/metrics'
import { 
  ChartBarIcon,
  ClockIcon,
  ArrowPathIcon,
  WrenchScrewdriverIcon
} from '@heroicons/vue/24/outline'

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, BarElement, Title, Tooltip, Legend, Filler)

const authStore = useAuthStore()
const serversStore = useServersStore()
const metricsStore = useMetricsStore()

const timeRanges = [
  { label: '15m', hours: 0.25 },
  { label: '1h', hours: 1 },
  { label: '6h', hours: 6 },
  { label: '24h', hours: 24 },
  { label: '7d', hours: 168 },
]

// Load persisted settings
const savedHours = localStorage.getItem('mcpx_dashboard_hours')
const savedAutoRefresh = localStorage.getItem('mcpx_dashboard_autorefresh')
const savedServer = localStorage.getItem('mcpx_dashboard_server')

const selectedHours = ref(savedHours ? parseFloat(savedHours) : 24)
const autoRefresh = ref(savedAutoRefresh === 'true')
const selectedServer = ref(savedServer || '')
const loading = ref(false)
const countdown = ref(60)
const toolLatencyTimeData = ref<{ labels: string[], datasets: any[] }>({ labels: [], datasets: [] })
const toolCallsTimeData = ref<{ labels: string[], datasets: any[] }>({ labels: [], datasets: [] })
let refreshTimer: ReturnType<typeof setInterval> | null = null
let countdownTimer: ReturnType<typeof setInterval> | null = null

// Toggle modes for cards
type LatencyMode = 'avg' | 'p50' | 'p95' | 'p99' | 'max'
type ThroughputMode = 'avg' | 'min' | 'max'

const selectedLatencyMode = ref<LatencyMode>('avg')
const selectedThroughputMode = ref<ThroughputMode>('avg')

const latencyModes = [
  { key: 'avg' as LatencyMode, label: 'avg' },
  { key: 'p50' as LatencyMode, label: 'p50' },
  { key: 'p95' as LatencyMode, label: 'p95' },
  { key: 'p99' as LatencyMode, label: 'p99' },
  { key: 'max' as LatencyMode, label: 'max' },
]

const throughputModes = [
  { key: 'avg' as ThroughputMode, label: 'avg' },
  { key: 'min' as ThroughputMode, label: 'min' },
  { key: 'max' as ThroughputMode, label: 'max' },
]

// Persist settings
watch(selectedHours, (val) => localStorage.setItem('mcpx_dashboard_hours', String(val)))
watch(autoRefresh, (val) => localStorage.setItem('mcpx_dashboard_autorefresh', String(val)))
watch(selectedServer, (val) => localStorage.setItem('mcpx_dashboard_server', val))

// Computed values from queryResponse
const queryData = computed(() => metricsStore.queryResponse?.data || [])

const totalRequests = computed(() => queryData.value.reduce((sum, d) => sum + d.count, 0))

const errorRate = computed(() => {
  const total = totalRequests.value
  const errors = queryData.value.reduce((sum, d) => sum + d.error_count, 0)
  return total > 0 ? (errors / total) * 100 : 0
})

const errorRateFormatted = computed(() => `${errorRate.value.toFixed(1)}%`)
const errorRateClass = computed(() => errorRate.value > 5 ? 'text-red-400' : errorRate.value > 1 ? 'text-yellow-400' : 'text-green-400')

const avgLatency = computed(() => {
  const data = queryData.value.filter(d => d.avg_latency_ms !== null)
  if (data.length === 0) return 0
  return data.reduce((sum, d) => sum + (d.avg_latency_ms || 0), 0) / data.length
})

const avgLatencyFormatted = computed(() => formatLatency(avgLatency.value))

const requestRate = computed(() => {
  const total = totalRequests.value
  const minutes = selectedHours.value * 60
  return minutes > 0 ? total / minutes : 0
})

const requestRateFormatted = computed(() => requestRate.value.toFixed(1))

// Computed values from summaryStats for toggle cards
const summary = computed(() => metricsStore.summaryStats)

const currentLatency = computed(() => {
  if (!summary.value) return avgLatency.value
  switch (selectedLatencyMode.value) {
    case 'avg': return summary.value.latency_avg ?? avgLatency.value
    case 'p50': return summary.value.latency_p50 ?? 0
    case 'p95': return summary.value.latency_p95 ?? 0
    case 'p99': return summary.value.latency_p99 ?? 0
    case 'max': return summary.value.latency_max ?? 0
    default: return avgLatency.value
  }
})

const currentLatencyFormatted = computed(() => formatLatency(currentLatency.value))
const latencyModeLabel = computed(() => {
  const mode = latencyModes.find(m => m.key === selectedLatencyMode.value)
  return mode?.label.toUpperCase() || 'AVG'
})

const currentThroughput = computed(() => {
  if (!summary.value) return requestRate.value
  switch (selectedThroughputMode.value) {
    case 'avg': return summary.value.throughput_avg ?? requestRate.value
    case 'min': return summary.value.throughput_min ?? 0
    case 'max': return summary.value.throughput_max ?? 0
    default: return requestRate.value
  }
})

const currentThroughputFormatted = computed(() => (currentThroughput.value ?? 0).toFixed(1))
const throughputModeLabel = computed(() => {
  const mode = throughputModes.find(m => m.key === selectedThroughputMode.value)
  return mode?.label.toUpperCase() || 'AVG'
})



// Chart data
const throughputData = computed(() => {
  const buckets = generateTimeBuckets()
  const data = queryData.value.filter(d => d.bucket).map(d => ({
    bucket: d.bucket!,
    value: d.count
  }))
  const filledData = fillBuckets(buckets, data, 0)
  
  return {
    labels: buckets.map(b => formatTime(b)),
    datasets: [{
      label: 'Requests',
      data: filledData,
      borderColor: '#9333ea',
      backgroundColor: 'rgba(147, 51, 234, 0.1)',
      fill: true,
      tension: 0.4,
      pointRadius: 1,
    }]
  }
})

const latencyData = computed(() => {
  const buckets = generateTimeBuckets()
  const data = queryData.value.filter(d => d.bucket && d.avg_latency_ms).map(d => ({
    bucket: d.bucket!,
    value: d.avg_latency_ms || 0
  }))
  const filledData = fillBuckets(buckets, data, 0)
  
  return {
    labels: buckets.map(b => formatTime(b)),
    datasets: [{
      label: 'Latency (ms)',
      data: filledData,
      borderColor: '#3b82f6',
      backgroundColor: 'rgba(59, 130, 246, 0.1)',
      fill: true,
      tension: 0.4,
      pointRadius: 1,
    }]
  }
})

const lineOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: { legend: { display: false } },
  scales: {
    x: { grid: { color: 'rgba(255,255,255,0.05)' }, ticks: { color: '#9ca3af', maxRotation: 0 } },
    y: { beginAtZero: true, grid: { color: 'rgba(255,255,255,0.05)' }, ticks: { color: '#9ca3af' } }
  }
}

const latencyOptions = {
  ...lineOptions,
  scales: {
    ...lineOptions.scales,
    y: { ...lineOptions.scales.y, ticks: { color: '#9ca3af', callback: (v: string | number) => typeof v === 'number' ? (v >= 1000 ? `${(v/1000).toFixed(1)}s` : `${v}ms`) : v } }
  }
}



function formatTime(iso: string): string {
  const date = new Date(iso)
  return selectedHours.value <= 24 
    ? date.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' })
    : date.toLocaleDateString('pt-BR', { day: '2-digit', month: '2-digit' })
}

function formatLatency(ms: number | null): string {
  if (ms === null) return '-'
  if (ms >= 1000) return `${(ms / 1000).toFixed(1)}s`
  return `${Math.round(ms)}ms`
}

// Generate complete time buckets from (now - hours) to now
function generateTimeBuckets(): string[] {
  const bucketSizeMinutes = selectedHours.value <= 1 ? 5 : selectedHours.value <= 6 ? 15 : 60
  const now = new Date()
  const start = new Date(now.getTime() - selectedHours.value * 60 * 60 * 1000)
  
  // Round start to bucket boundary
  start.setMinutes(Math.floor(start.getMinutes() / bucketSizeMinutes) * bucketSizeMinutes, 0, 0)
  
  const buckets: string[] = []
  const current = new Date(start)
  
  while (current <= now) {
    buckets.push(current.toISOString())
    current.setMinutes(current.getMinutes() + bucketSizeMinutes)
  }
  
  return buckets
}

// Fill data into time buckets, using 0 for missing buckets
function fillBuckets<T>(
  buckets: string[], 
  data: { bucket: string; value: T }[],
  defaultValue: T
): T[] {
  const dataMap = new Map(data.map(d => [new Date(d.bucket).toISOString(), d.value]))
  return buckets.map(bucket => dataMap.get(bucket) ?? defaultValue)
}



function setTimeRange(hours: number) {
  selectedHours.value = hours
  refreshAll()
}

function toggleAutoRefresh() {
  autoRefresh.value = !autoRefresh.value
  if (autoRefresh.value) {
    countdown.value = 60
    refreshTimer = setInterval(() => { countdown.value = 60; refreshAll() }, 60000)
    countdownTimer = setInterval(() => { if (countdown.value > 0) countdown.value-- }, 1000)
    refreshAll()
  } else {
    if (refreshTimer) clearInterval(refreshTimer)
    if (countdownTimer) clearInterval(countdownTimer)
    refreshTimer = null
    countdownTimer = null
  }
}

async function refreshAll() {
  loading.value = true
  try {
    const filters = selectedServer.value ? [{ field: 'target_name', op: 'eq', value: selectedServer.value }] : []
    
    await Promise.all([
      metricsStore.queryMetrics({
        time_range_hours: selectedHours.value,
        group_by: ['time_bucket'],
        bucket_size: selectedHours.value <= 1 ? '5m' : selectedHours.value <= 6 ? '15m' : '1h',
        filters
      }),
      metricsStore.fetchByTarget(selectedHours.value),
      metricsStore.fetchTodayMetrics(),
      metricsStore.fetchSummary(selectedHours.value, selectedServer.value || undefined),
      fetchToolCallsTimeData(),
      fetchToolLatencyTimeData()
    ])
  } finally {
    loading.value = false
  }
}

async function fetchToolLatencyTimeData() {
  try {
    const filters = selectedServer.value ? [{ field: 'target_name', op: 'eq', value: selectedServer.value }] : []
    const response = await import('@/api/client').then(m => m.default.post('/metrics/query', {
      time_range_hours: selectedHours.value,
      group_by: ['time_bucket', 'target_name', 'tool_name'],
      bucket_size: selectedHours.value <= 1 ? '5m' : selectedHours.value <= 6 ? '15m' : '1h',
      filters
    }))
    
    const data = (response.data.data || []).filter((d: { tool: string | null }) => d.tool)
    
    // Use complete time buckets from now - selectedHours to now
    const buckets = generateTimeBuckets()
    
    // Create server:tool keys and count totals
    const toolCounts = new Map<string, number>()
    data.forEach((d: { target_name?: string, tool: string, count: number }) => {
      const key = d.target_name ? `${d.target_name}:${d.tool}` : d.tool
      toolCounts.set(key, (toolCounts.get(key) || 0) + d.count)
    })
    const topTools = [...toolCounts.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 8)
      .map(([name]) => name)
    
    // Build datasets for each server:tool using avg_latency_ms
    const datasets = topTools.map((toolKey, idx) => {
      const toolData = data.filter((d: { target_name?: string, tool: string }) => {
        const key = d.target_name ? `${d.target_name}:${d.tool}` : d.tool
        return key === toolKey
      })
      const latencies = buckets.map(bucket => {
        const bucketTime = new Date(bucket).getTime()
        const match = toolData.find((d: { bucket: string }) => new Date(d.bucket).getTime() === bucketTime)
        return match ? match.avg_latency_ms || 0 : 0
      })
      return {
        label: toolKey,
        data: latencies,
        borderColor: TOOL_COLORS[idx % TOOL_COLORS.length],
        backgroundColor: 'transparent',
        tension: 0.3,
        pointRadius: 2,
      }
    })
    
    toolLatencyTimeData.value = {
      labels: buckets.map((b: string) => formatTime(b)),
      datasets
    }
  } catch {
    toolLatencyTimeData.value = { labels: [], datasets: [] }
  }
}

// Colors for different tool lines
const TOOL_COLORS = ['#f97316', '#06b6d4', '#8b5cf6', '#22c55e', '#ec4899', '#eab308', '#3b82f6', '#ef4444']

async function fetchToolCallsTimeData() {
  try {
    const filters = selectedServer.value ? [{ field: 'target_name', op: 'eq', value: selectedServer.value }] : []
    const response = await import('@/api/client').then(m => m.default.post('/metrics/query', {
      time_range_hours: selectedHours.value,
      group_by: ['time_bucket', 'target_name', 'tool_name'],
      bucket_size: selectedHours.value <= 1 ? '5m' : selectedHours.value <= 6 ? '15m' : '1h',
      filters
    }))
    
    const data = (response.data.data || []).filter((d: { tool: string | null }) => d.tool)
    
    // Use complete time buckets from now - selectedHours to now
    const buckets = generateTimeBuckets()
    
    // Create server:tool keys and count totals
    const toolCounts = new Map<string, number>()
    data.forEach((d: { target_name?: string, tool: string, count: number }) => {
      const key = d.target_name ? `${d.target_name}:${d.tool}` : d.tool
      toolCounts.set(key, (toolCounts.get(key) || 0) + d.count)
    })
    const topTools = [...toolCounts.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 8)
      .map(([name]) => name)
    
    // Build datasets for each server:tool
    const datasets = topTools.map((toolKey, idx) => {
      const toolData = data.filter((d: { target_name?: string, tool: string }) => {
        const key = d.target_name ? `${d.target_name}:${d.tool}` : d.tool
        return key === toolKey
      })
      const counts = buckets.map(bucket => {
        const bucketTime = new Date(bucket).getTime()
        const match = toolData.find((d: { bucket: string }) => new Date(d.bucket).getTime() === bucketTime)
        return match ? match.count : 0
      })
      return {
        label: toolKey,
        data: counts,
        borderColor: TOOL_COLORS[idx % TOOL_COLORS.length],
        backgroundColor: 'transparent',
        tension: 0.3,
        pointRadius: 2,
      }
    })
    
    toolCallsTimeData.value = {
      labels: buckets.map((b: string) => formatTime(b)),
      datasets
    }
  } catch {
    toolCallsTimeData.value = { labels: [], datasets: [] }
  }
}

const toolCallsTimeOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: { 
    legend: { 
      display: true, 
      position: 'top' as const,
      labels: { color: '#9ca3af', boxWidth: 12, padding: 10 }
    } 
  },
  scales: {
    x: { grid: { color: 'rgba(255,255,255,0.05)' }, ticks: { color: '#9ca3af', maxRotation: 0 } },
    y: { beginAtZero: true, grid: { color: 'rgba(255,255,255,0.05)' }, ticks: { color: '#9ca3af' } }
  }
}

const toolLatencyTimeOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: { 
    legend: { 
      display: true, 
      position: 'top' as const,
      labels: { color: '#9ca3af', boxWidth: 12, padding: 10 }
    } 
  },
  scales: {
    x: { grid: { color: 'rgba(255,255,255,0.05)' }, ticks: { color: '#9ca3af', maxRotation: 0 } },
    y: { 
      beginAtZero: true, 
      grid: { color: 'rgba(255,255,255,0.05)' }, 
      ticks: { 
        color: '#9ca3af',
        callback: (v: string | number) => typeof v === 'number' ? (v >= 1000 ? `${(v/1000).toFixed(1)}s` : `${v}ms`) : v
      } 
    }
  }
}

onMounted(() => {
  serversStore.fetchServers()
  refreshAll()
  if (autoRefresh.value) {
    countdown.value = 60
    refreshTimer = setInterval(() => { countdown.value = 60; refreshAll() }, 60000)
    countdownTimer = setInterval(() => { if (countdown.value > 0) countdown.value-- }, 1000)
  }
})

onUnmounted(() => {
  if (refreshTimer) clearInterval(refreshTimer)
  if (countdownTimer) clearInterval(countdownTimer)
})
</script>
