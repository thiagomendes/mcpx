import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface AlertRule {
    id: string
    user_id: string
    name: string
    alert_type: 'threshold' | 'spike' | 'no_data'
    metric: 'error_rate' | 'avg_latency' | 'request_count' | 'p95_latency'
    scope_type: 'all' | 'server' | 'gateway'
    scope_id?: string
    operator: 'above' | 'below'
    threshold: number
    duration_minutes: number
    notify_dashboard: boolean
    enabled: boolean
    created_at: string
    updated_at: string
}

export interface ActiveAlert {
    id: string
    rule_id: string
    rule_name: string
    alert_type: string
    metric: string
    scope_type: string
    scope_id?: string
    scope_name?: string
    triggered_at: string
    trigger_value: number
    threshold: number
    status: 'triggered' | 'acknowledged'
}

export interface AlertHistory {
    id: string
    rule_id: string
    triggered_at: string
    resolved_at?: string
    trigger_value: number
    status: 'triggered' | 'resolved' | 'acknowledged'
}

export interface CreateAlertRule {
    name: string
    alert_type: string
    metric: string
    scope_type: string
    scope_id?: string
    operator: string
    threshold: number
    duration_minutes: number
    notify_dashboard?: boolean
}

export interface UpdateAlertRule {
    name?: string
    alert_type?: string
    metric?: string
    scope_type?: string
    scope_id?: string
    operator?: string
    threshold?: number
    duration_minutes?: number
    notify_dashboard?: boolean
    enabled?: boolean
}

export const useAlertsStore = defineStore('alerts', () => {
    const rules = ref<AlertRule[]>([])
    const activeAlerts = ref<ActiveAlert[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    async function fetchRules() {
        loading.value = true
        error.value = null
        try {
            const response = await api.get<AlertRule[]>('/alerts')
            rules.value = response.data
        } catch {
            error.value = 'Failed to fetch alert rules'
            rules.value = []
        } finally {
            loading.value = false
        }
    }

    async function fetchActiveAlerts() {
        try {
            const response = await api.get<ActiveAlert[]>('/alerts/active')
            activeAlerts.value = response.data
        } catch {
            activeAlerts.value = []
        }
    }

    async function createRule(input: CreateAlertRule): Promise<AlertRule | null> {
        loading.value = true
        error.value = null
        try {
            const response = await api.post<AlertRule>('/alerts', input)
            rules.value.unshift(response.data)
            return response.data
        } catch {
            error.value = 'Failed to create alert rule'
            return null
        } finally {
            loading.value = false
        }
    }

    async function updateRule(id: string, input: UpdateAlertRule): Promise<AlertRule | null> {
        loading.value = true
        error.value = null
        try {
            const response = await api.put<AlertRule>(`/alerts/${id}`, input)
            const index = rules.value.findIndex(r => r.id === id)
            if (index !== -1) {
                rules.value[index] = response.data
            }
            return response.data
        } catch {
            error.value = 'Failed to update alert rule'
            return null
        } finally {
            loading.value = false
        }
    }

    async function deleteRule(id: string): Promise<boolean> {
        loading.value = true
        error.value = null
        try {
            await api.delete(`/alerts/${id}`)
            rules.value = rules.value.filter(r => r.id !== id)
            return true
        } catch {
            error.value = 'Failed to delete alert rule'
            return false
        } finally {
            loading.value = false
        }
    }

    async function toggleRule(id: string, enabled: boolean): Promise<boolean> {
        const result = await updateRule(id, { enabled })
        return result !== null
    }

    async function acknowledgeAlert(id: string): Promise<boolean> {
        try {
            await api.post(`/alerts/${id}/acknowledge`)
            const alert = activeAlerts.value.find(a => a.id === id)
            if (alert) {
                alert.status = 'acknowledged'
            }
            return true
        } catch {
            return false
        }
    }

    return {
        rules,
        activeAlerts,
        loading,
        error,
        fetchRules,
        fetchActiveAlerts,
        createRule,
        updateRule,
        deleteRule,
        toggleRule,
        acknowledgeAlert,
    }
})
