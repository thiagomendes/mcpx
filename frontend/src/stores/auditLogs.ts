import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface AuditLogEntry {
    id: string
    time: string
    user_id: string | null
    action: string
    resource_type: string
    resource_id: string | null
    resource_name: string | null
}

export interface AuditLogDetail extends AuditLogEntry {
    details: object | null
    ip_address: string | null
    user_agent: string | null
}

export interface AuditLogListResponse {
    data: AuditLogEntry[]
    total: number
    limit: number
    offset: number
}

export interface AuditLogFilters {
    action?: string
    resource_type?: string
    user_id?: string
    hours?: number
    limit?: number
    offset?: number
}

export const useAuditLogsStore = defineStore('auditLogs', () => {
    const logs = ref<AuditLogEntry[]>([])
    const selectedLog = ref<AuditLogDetail | null>(null)
    const total = ref(0)
    const loading = ref(false)
    const error = ref<string | null>(null)
    const currentFilters = ref<AuditLogFilters>({
        hours: 168, // 7 days
        limit: 50,
        offset: 0,
    })

    async function fetchLogs(filters?: AuditLogFilters) {
        loading.value = true
        error.value = null

        const params = { ...currentFilters.value, ...filters }
        currentFilters.value = params

        try {
            const queryParams = new URLSearchParams()
            if (params.action) queryParams.append('action', params.action)
            if (params.resource_type) queryParams.append('resource_type', params.resource_type)
            if (params.user_id) queryParams.append('user_id', params.user_id)
            if (params.hours) queryParams.append('hours', String(params.hours))
            if (params.limit) queryParams.append('limit', String(params.limit))
            if (params.offset) queryParams.append('offset', String(params.offset))

            const response = await api.get<AuditLogListResponse>(`/audit-logs?${queryParams.toString()}`)
            logs.value = response.data.data
            total.value = response.data.total
        } catch {
            error.value = 'Failed to fetch audit logs'
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
            const response = await api.get<AuditLogDetail>(`/audit-logs/${id}`)
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

    // Helper function to format action
    function formatAction(action: string): string {
        return action.replace('.', ' → ')
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
        formatAction,
    }
})
