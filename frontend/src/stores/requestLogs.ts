import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface RequestLogEntry {
    id: string
    time: string
    target_type: string
    target_id: string
    target_name: string
    method: string | null
    tool_name: string | null
    status_code: number | null
    latency_ms: number | null
    success: boolean
    error_message: string | null
}

export interface RequestLogDetail extends RequestLogEntry {
    request_body: object | null
    response_body: object | null
    session_id: string | null
}

export interface RequestLogListResponse {
    data: RequestLogEntry[]
    total: number
    limit: number
    offset: number
}

export interface RequestLogFilters {
    target_name?: string
    method?: string
    tool_name?: string
    success?: boolean
    hours?: number
    limit?: number
    offset?: number
}

export const useRequestLogsStore = defineStore('requestLogs', () => {
    const logs = ref<RequestLogEntry[]>([])
    const selectedLog = ref<RequestLogDetail | null>(null)
    const total = ref(0)
    const loading = ref(false)
    const error = ref<string | null>(null)
    const currentFilters = ref<RequestLogFilters>({
        hours: 24,
        limit: 50,
        offset: 0,
    })

    async function fetchLogs(filters?: RequestLogFilters) {
        loading.value = true
        error.value = null

        const params = { ...currentFilters.value, ...filters }
        currentFilters.value = params

        try {
            const queryParams = new URLSearchParams()
            if (params.target_name) queryParams.append('target_name', params.target_name)
            if (params.method) queryParams.append('method', params.method)
            if (params.tool_name) queryParams.append('tool_name', params.tool_name)
            if (params.success !== undefined) queryParams.append('success', String(params.success))
            if (params.hours) queryParams.append('hours', String(params.hours))
            if (params.limit) queryParams.append('limit', String(params.limit))
            if (params.offset) queryParams.append('offset', String(params.offset))

            const response = await api.get<RequestLogListResponse>(`/request-logs?${queryParams.toString()}`)
            logs.value = response.data.data
            total.value = response.data.total
        } catch {
            error.value = 'Failed to fetch request logs'
            logs.value = []
            total.value = 0
        } finally {
            loading.value = false
        }
    }

    async function fetchLogDetail(id: string) {
        loading.value = true
        error.value = null

        try {
            const response = await api.get<RequestLogDetail>(`/request-logs/${id}`)
            selectedLog.value = response.data
        } catch {
            error.value = 'Failed to fetch log details'
            selectedLog.value = null
        } finally {
            loading.value = false
        }
    }

    function nextPage() {
        const newOffset = (currentFilters.value.offset || 0) + (currentFilters.value.limit || 50)
        if (newOffset < total.value) {
            fetchLogs({ offset: newOffset })
        }
    }

    function prevPage() {
        const newOffset = Math.max(0, (currentFilters.value.offset || 0) - (currentFilters.value.limit || 50))
        fetchLogs({ offset: newOffset })
    }

    function clearSelection() {
        selectedLog.value = null
    }

    return {
        logs,
        selectedLog,
        total,
        loading,
        error,
        currentFilters,
        fetchLogs,
        fetchLogDetail,
        nextPage,
        prevPage,
        clearSelection,
    }
})
