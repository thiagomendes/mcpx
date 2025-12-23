import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface TodayMetrics {
    servers: number
    gateways: number
}

export interface HourlyStat {
    bucket: string
    total: number
    success_count: number
    avg_latency_ms: number | null
}

export interface TargetMetrics {
    target_type: string
    target_id: string
    target_name: string
    total: number
    success_count: number
    avg_latency_ms: number | null
}

export interface QueryFilter {
    field: string
    op: string
    value?: string
    values?: string[]
}

export interface MetricsQuery {
    time_range_hours: number
    group_by: string[]
    bucket_size: string
    filters: QueryFilter[]
}

export interface QueryResult {
    bucket: string | null
    target_type: string | null
    target_name: string | null
    tool: string | null
    count: number
    success_count: number
    error_count: number
    avg_latency_ms: number | null
    success_rate: number | null
}

export interface QueryResponse {
    data: QueryResult[]
    meta: {
        total_count: number
        time_range_hours: number
        bucket_size: string
    }
}

export interface SummaryStats {
    // Latency (ms)
    latency_avg: number | null
    latency_p50: number | null
    latency_p95: number | null
    latency_p99: number | null
    latency_max: number | null
    // Throughput (req/min)
    throughput_avg: number | null
    throughput_min: number | null
    throughput_max: number | null
    // Counts
    total_requests: number
    error_count: number
    error_rate: number
}

export const useMetricsStore = defineStore('metrics', () => {
    const todayMetrics = ref<TodayMetrics>({ servers: 0, gateways: 0 })
    const hourlyStats = ref<HourlyStat[]>([])
    const targetMetrics = ref<TargetMetrics[]>([])
    const queryResponse = ref<QueryResponse | null>(null)
    const summaryStats = ref<SummaryStats | null>(null)
    const loading = ref(false)
    const error = ref<string | null>(null)

    async function fetchTodayMetrics() {
        loading.value = true
        error.value = null
        try {
            const response = await api.get('/metrics/today')
            todayMetrics.value = response.data
        } catch {
            error.value = 'Failed to fetch metrics'
            todayMetrics.value = { servers: 0, gateways: 0 }
        } finally {
            loading.value = false
        }
    }

    async function fetchHourlyStats(hours: number = 24) {
        loading.value = true
        error.value = null
        try {
            const response = await api.get(`/metrics/hourly?hours=${hours}`)
            hourlyStats.value = response.data
        } catch {
            error.value = 'Failed to fetch hourly stats'
            hourlyStats.value = []
        } finally {
            loading.value = false
        }
    }

    async function fetchByTarget(hours: number = 24) {
        try {
            const response = await api.get(`/metrics/by-target?hours=${hours}`)
            targetMetrics.value = response.data
        } catch {
            targetMetrics.value = []
        }
    }

    async function queryMetrics(query: MetricsQuery) {
        loading.value = true
        error.value = null
        try {
            const response = await api.post('/metrics/query', query)
            queryResponse.value = response.data
        } catch {
            error.value = 'Failed to query metrics'
            queryResponse.value = null
        } finally {
            loading.value = false
        }
    }

    async function fetchSummary(hours: number = 24, targetName?: string) {
        try {
            const params = new URLSearchParams()
            params.append('hours', String(hours))
            if (targetName) params.append('target_name', targetName)
            const response = await api.get(`/metrics/summary?${params.toString()}`)
            summaryStats.value = response.data
        } catch {
            summaryStats.value = null
        }
    }

    return {
        todayMetrics,
        hourlyStats,
        targetMetrics,
        queryResponse,
        summaryStats,
        loading,
        error,
        fetchTodayMetrics,
        fetchHourlyStats,
        fetchByTarget,
        queryMetrics,
        fetchSummary,
    }
})
