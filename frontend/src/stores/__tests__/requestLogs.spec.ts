import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useRequestLogsStore } from '@/stores/requestLogs'
import api from '@/api/client'

vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn(),
    },
}))

describe('Request Logs Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    describe('initial state', () => {
        it('starts with default values', () => {
            const store = useRequestLogsStore()
            expect(store.logs).toEqual([])
            expect(store.selectedLog).toBeNull()
            expect(store.total).toBe(0)
            expect(store.loading).toBe(false)
            expect(store.error).toBeNull()
            expect(store.currentFilters).toEqual({
                hours: 24,
                limit: 50,
                offset: 0,
            })
        })
    })

    describe('fetchLogs', () => {
        it('fetches logs with default filters', async () => {
            const mockResponse = {
                data: [
                    {
                        id: '1',
                        time: '2024-01-01T00:00:00Z',
                        target_type: 'server',
                        target_id: 'server1',
                        target_name: 'My Server',
                        method: 'tools/call',
                        tool_name: 'get_weather',
                        status_code: 200,
                        latency_ms: 50,
                        success: true,
                        error_message: null,
                    },
                ],
                total: 1,
                limit: 50,
                offset: 0,
            }
            vi.mocked(api.get).mockResolvedValue({ data: mockResponse })

            const store = useRequestLogsStore()
            await store.fetchLogs()

            expect(api.get).toHaveBeenCalledWith(expect.stringContaining('/request-logs?'))
            expect(store.logs).toEqual(mockResponse.data)
            expect(store.total).toBe(1)
            expect(store.loading).toBe(false)
        })

        it('fetches logs with custom filters', async () => {
            const mockResponse = { data: [], total: 0, limit: 50, offset: 0 }
            vi.mocked(api.get).mockResolvedValue({ data: mockResponse })

            const store = useRequestLogsStore()
            await store.fetchLogs({
                target_name: 'server-1',
                method: 'tools/call',
                success: true,
                hours: 48,
            })

            expect(api.get).toHaveBeenCalledWith(expect.stringMatching(/target_name=server-1/))
            expect(api.get).toHaveBeenCalledWith(expect.stringMatching(/method=tools%2Fcall/))
            expect(api.get).toHaveBeenCalledWith(expect.stringMatching(/success=true/))
        })

        it('handles fetch error', async () => {
            vi.mocked(api.get).mockRejectedValue(new Error('Network error'))

            const store = useRequestLogsStore()
            await store.fetchLogs()

            expect(store.error).toBe('Failed to fetch request logs')
            expect(store.logs).toEqual([])
            expect(store.total).toBe(0)
        })
    })

    describe('fetchLogDetail', () => {
        it('fetches log detail', async () => {
            const mockDetail = {
                id: '1',
                time: '2024-01-01T00:00:00Z',
                target_type: 'server',
                target_id: 'server1',
                target_name: 'My Server',
                method: 'tools/call',
                tool_name: 'get_weather',
                status_code: 200,
                latency_ms: 50,
                success: true,
                error_message: null,
                request_body: { jsonrpc: '2.0', method: 'tools/call' },
                response_body: { result: 'success' },
                session_id: 'session123',
            }
            vi.mocked(api.get).mockResolvedValue({ data: mockDetail })

            const store = useRequestLogsStore()
            await store.fetchLogDetail('1')

            expect(api.get).toHaveBeenCalledWith('/request-logs/1')
            expect(store.selectedLog).toEqual(mockDetail)
            expect(store.loading).toBe(false)
        })

        it('handles fetch detail error', async () => {
            vi.mocked(api.get).mockRejectedValue(new Error('Not found'))

            const store = useRequestLogsStore()
            await store.fetchLogDetail('999')

            expect(store.error).toBe('Failed to fetch log details')
            expect(store.selectedLog).toBeNull()
        })
    })

    describe('pagination', () => {
        it('nextPage increases offset when more data available', async () => {
            const mockResponse = {
                data: Array(50).fill({
                    id: '1',
                    time: '2024-01-01T00:00:00Z',
                    target_type: 'server',
                    target_id: 'server1',
                    target_name: 'My Server',
                    method: 'tools/call',
                    tool_name: null,
                    status_code: 200,
                    latency_ms: 50,
                    success: true,
                    error_message: null,
                }),
                total: 100,
                limit: 50,
                offset: 0,
            }
            vi.mocked(api.get).mockResolvedValue({ data: mockResponse })

            const store = useRequestLogsStore()
            await store.fetchLogs()
            expect(store.total).toBe(100)

            // Go to next page
            store.nextPage()

            expect(api.get).toHaveBeenLastCalledWith(expect.stringMatching(/offset=50/))
        })

        it('prevPage decreases offset', async () => {
            const mockResponse = { data: [], total: 100, limit: 50, offset: 0 }
            vi.mocked(api.get).mockResolvedValue({ data: mockResponse })

            const store = useRequestLogsStore()
            store.currentFilters.offset = 50

            store.prevPage()

            // Check that offset was reset to 0 in the filters
            expect(store.currentFilters.offset).toBe(0)
        })

        it('prevPage does not go below 0', async () => {
            const mockResponse = { data: [], total: 100, limit: 50, offset: 0 }
            vi.mocked(api.get).mockResolvedValue({ data: mockResponse })

            const store = useRequestLogsStore()
            store.currentFilters.offset = 0

            store.prevPage()

            // Offset should stay at 0
            expect(store.currentFilters.offset).toBe(0)
        })
    })

    describe('clearSelection', () => {
        it('clears selected log', async () => {
            const mockDetail = {
                id: '1',
                time: '2024-01-01T00:00:00Z',
                target_type: 'server',
                target_id: 'server1',
                target_name: 'My Server',
                method: 'tools/call',
                tool_name: null,
                status_code: 200,
                latency_ms: 50,
                success: true,
                error_message: null,
                request_body: null,
                response_body: null,
                session_id: null,
            }
            vi.mocked(api.get).mockResolvedValue({ data: mockDetail })

            const store = useRequestLogsStore()
            await store.fetchLogDetail('1')
            expect(store.selectedLog).not.toBeNull()

            store.clearSelection()

            expect(store.selectedLog).toBeNull()
        })
    })
})
