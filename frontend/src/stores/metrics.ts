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

export const useMetricsStore = defineStore('metrics', () => {
    const todayMetrics = ref<TodayMetrics>({ servers: 0, gateways: 0 })
    const hourlyStats = ref<HourlyStat[]>([])
    const targetMetrics = ref<TargetMetrics[]>([])
    const queryResponse = ref<QueryResponse | null>(null)
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

    return {
        todayMetrics,
        hourlyStats,
        targetMetrics,
        queryResponse,
        loading,
        error,
        fetchTodayMetrics,
        fetchHourlyStats,
        fetchByTarget,
        queryMetrics,
    }
})
