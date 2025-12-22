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

export const useMetricsStore = defineStore('metrics', () => {
    const todayMetrics = ref<TodayMetrics>({ servers: 0, gateways: 0 })
    const hourlyStats = ref<HourlyStat[]>([])
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
            // Return zeros on error (graceful degradation)
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

    return {
        todayMetrics,
        hourlyStats,
        loading,
        error,
        fetchTodayMetrics,
        fetchHourlyStats,
    }
})
