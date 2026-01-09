import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useMetricsStore } from '@/stores/metrics'
import api from '@/api/client'

vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn(),
    },
}))

describe('Metrics Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    describe('fetchTodayMetrics', () => {
        it('fetches today metrics and updates state', async () => {
            const mockMetrics = { servers: 42, gateways: 15 }
            vi.mocked(api.get).mockResolvedValue({ data: mockMetrics })

            const store = useMetricsStore()
            await store.fetchTodayMetrics()

            expect(api.get).toHaveBeenCalledWith('/metrics/today')
            expect(store.todayMetrics.servers).toBe(42)
            expect(store.todayMetrics.gateways).toBe(15)
            expect(store.loading).toBe(false)
            expect(store.error).toBeNull()
        })

        it('handles fetch error with graceful degradation', async () => {
            vi.mocked(api.get).mockRejectedValue(new Error('Network error'))

            const store = useMetricsStore()
            await store.fetchTodayMetrics()

            expect(store.error).toBe('Failed to fetch metrics')
            expect(store.todayMetrics.servers).toBe(0)
            expect(store.todayMetrics.gateways).toBe(0)
            expect(store.loading).toBe(false)
        })
    })

    describe('fetchHourlyStats', () => {
        it('fetches hourly stats with default 24 hours', async () => {
            const mockStats = [
                { bucket: '2024-01-15T10:00:00Z', total: 23, success_count: 22, avg_latency_ms: 145 },
                { bucket: '2024-01-15T11:00:00Z', total: 31, success_count: 30, avg_latency_ms: 132 },
            ]
            vi.mocked(api.get).mockResolvedValue({ data: mockStats })

            const store = useMetricsStore()
            await store.fetchHourlyStats()

            expect(api.get).toHaveBeenCalledWith('/metrics/hourly?hours=24')
            expect(store.hourlyStats.length).toBe(2)
            expect(store.hourlyStats[0].total).toBe(23)
        })

        it('fetches hourly stats with custom hours', async () => {
            const mockStats = [{ bucket: '2024-01-15T12:00:00Z', total: 10, success_count: 10, avg_latency_ms: 100 }]
            vi.mocked(api.get).mockResolvedValue({ data: mockStats })

            const store = useMetricsStore()
            await store.fetchHourlyStats(6)

            expect(api.get).toHaveBeenCalledWith('/metrics/hourly?hours=6')
        })

        it('handles fetch error', async () => {
            vi.mocked(api.get).mockRejectedValue(new Error('Network error'))

            const store = useMetricsStore()
            await store.fetchHourlyStats()

            expect(store.error).toBe('Failed to fetch hourly stats')
            expect(store.hourlyStats).toEqual([])
        })
    })
})
