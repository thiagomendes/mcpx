import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useAlertsStore } from '@/stores/alerts'
import api from '@/api/client'

vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn(),
        post: vi.fn(),
        put: vi.fn(),
        delete: vi.fn(),
    },
}))

describe('Alerts Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    describe('initial state', () => {
        it('starts with empty rules and alerts', () => {
            const store = useAlertsStore()
            expect(store.rules).toEqual([])
            expect(store.activeAlerts).toEqual([])
            expect(store.loading).toBe(false)
            expect(store.error).toBeNull()
        })
    })

    describe('fetchRules', () => {
        it('fetches rules and updates state', async () => {
            const mockRules = [
                {
                    id: '1',
                    user_id: 'user1',
                    name: 'High Error Rate',
                    alert_type: 'threshold',
                    metric: 'error_rate',
                    scope_type: 'all',
                    operator: 'above',
                    threshold: 0.1,
                    duration_minutes: 5,
                    notify_dashboard: true,
                    enabled: true,
                    created_at: '2024-01-01T00:00:00Z',
                    updated_at: '2024-01-01T00:00:00Z',
                },
            ]
            vi.mocked(api.get).mockResolvedValue({ data: mockRules })

            const store = useAlertsStore()
            await store.fetchRules()

            expect(api.get).toHaveBeenCalledWith('/alerts')
            expect(store.rules).toEqual(mockRules)
            expect(store.loading).toBe(false)
        })

        it('handles fetch error', async () => {
            vi.mocked(api.get).mockRejectedValue(new Error('Network error'))

            const store = useAlertsStore()
            await store.fetchRules()

            expect(store.error).toBe('Failed to fetch alert rules')
            expect(store.rules).toEqual([])
        })
    })

    describe('fetchActiveAlerts', () => {
        it('fetches active alerts', async () => {
            const mockAlerts = [
                {
                    id: 'alert1',
                    rule_id: 'rule1',
                    rule_name: 'High Error Rate',
                    alert_type: 'threshold',
                    metric: 'error_rate',
                    scope_type: 'server',
                    scope_id: 'server1',
                    scope_name: 'My Server',
                    triggered_at: '2024-01-01T00:00:00Z',
                    trigger_value: 0.15,
                    threshold: 0.1,
                    status: 'triggered',
                },
            ]
            vi.mocked(api.get).mockResolvedValue({ data: mockAlerts })

            const store = useAlertsStore()
            await store.fetchActiveAlerts()

            expect(api.get).toHaveBeenCalledWith('/alerts/active')
            expect(store.activeAlerts).toEqual(mockAlerts)
        })

        it('handles error by setting empty array', async () => {
            vi.mocked(api.get).mockRejectedValue(new Error('Network error'))

            const store = useAlertsStore()
            await store.fetchActiveAlerts()

            expect(store.activeAlerts).toEqual([])
        })
    })

    describe('createRule', () => {
        it('creates rule and adds to state', async () => {
            const newRule = {
                id: '2',
                user_id: 'user1',
                name: 'New Rule',
                alert_type: 'threshold' as const,
                metric: 'avg_latency' as const,
                scope_type: 'all' as const,
                operator: 'above' as const,
                threshold: 1000,
                duration_minutes: 10,
                notify_dashboard: true,
                enabled: true,
                created_at: '2024-01-02T00:00:00Z',
                updated_at: '2024-01-02T00:00:00Z',
            }
            vi.mocked(api.post).mockResolvedValue({ data: newRule })

            const store = useAlertsStore()
            const result = await store.createRule({
                name: 'New Rule',
                alert_type: 'threshold',
                metric: 'avg_latency',
                scope_type: 'all',
                operator: 'above',
                threshold: 1000,
                duration_minutes: 10,
            })

            expect(api.post).toHaveBeenCalledWith('/alerts', expect.any(Object))
            expect(result).toEqual(newRule)
            expect(store.rules[0]).toEqual(newRule)
        })

        it('returns null on error', async () => {
            vi.mocked(api.post).mockRejectedValue(new Error('Failed'))

            const store = useAlertsStore()
            const result = await store.createRule({
                name: 'New Rule',
                alert_type: 'threshold',
                metric: 'avg_latency',
                scope_type: 'all',
                operator: 'above',
                threshold: 1000,
                duration_minutes: 10,
            })

            expect(result).toBeNull()
            expect(store.error).toBe('Failed to create alert rule')
        })
    })

    describe('updateRule', () => {
        it('updates rule in state', async () => {
            const existingRule = {
                id: '1',
                user_id: 'user1',
                name: 'Old Name',
                alert_type: 'threshold' as const,
                metric: 'error_rate' as const,
                scope_type: 'all' as const,
                operator: 'above' as const,
                threshold: 0.1,
                duration_minutes: 5,
                notify_dashboard: true,
                enabled: true,
                created_at: '2024-01-01T00:00:00Z',
                updated_at: '2024-01-01T00:00:00Z',
            }
            const updatedRule = { ...existingRule, name: 'New Name' }

            vi.mocked(api.get).mockResolvedValue({ data: [existingRule] })
            vi.mocked(api.put).mockResolvedValue({ data: updatedRule })

            const store = useAlertsStore()
            await store.fetchRules()
            const result = await store.updateRule('1', { name: 'New Name' })

            expect(api.put).toHaveBeenCalledWith('/alerts/1', { name: 'New Name' })
            expect(result?.name).toBe('New Name')
            expect(store.rules[0].name).toBe('New Name')
        })

        it('returns null on error', async () => {
            vi.mocked(api.put).mockRejectedValue(new Error('Failed'))

            const store = useAlertsStore()
            const result = await store.updateRule('1', { name: 'New Name' })

            expect(result).toBeNull()
            expect(store.error).toBe('Failed to update alert rule')
        })
    })

    describe('deleteRule', () => {
        it('deletes rule and removes from state', async () => {
            const rule = {
                id: '1',
                user_id: 'user1',
                name: 'To Delete',
                alert_type: 'threshold' as const,
                metric: 'error_rate' as const,
                scope_type: 'all' as const,
                operator: 'above' as const,
                threshold: 0.1,
                duration_minutes: 5,
                notify_dashboard: true,
                enabled: true,
                created_at: '2024-01-01T00:00:00Z',
                updated_at: '2024-01-01T00:00:00Z',
            }
            vi.mocked(api.get).mockResolvedValue({ data: [rule] })
            vi.mocked(api.delete).mockResolvedValue({})

            const store = useAlertsStore()
            await store.fetchRules()
            expect(store.rules.length).toBe(1)

            const result = await store.deleteRule('1')

            expect(result).toBe(true)
            expect(api.delete).toHaveBeenCalledWith('/alerts/1')
            expect(store.rules.length).toBe(0)
        })

        it('returns false on error', async () => {
            vi.mocked(api.delete).mockRejectedValue(new Error('Failed'))

            const store = useAlertsStore()
            const result = await store.deleteRule('1')

            expect(result).toBe(false)
            expect(store.error).toBe('Failed to delete alert rule')
        })
    })

    describe('toggleRule', () => {
        it('toggles rule enabled status', async () => {
            const rule = {
                id: '1',
                user_id: 'user1',
                name: 'Test',
                alert_type: 'threshold' as const,
                metric: 'error_rate' as const,
                scope_type: 'all' as const,
                operator: 'above' as const,
                threshold: 0.1,
                duration_minutes: 5,
                notify_dashboard: true,
                enabled: true,
                created_at: '2024-01-01T00:00:00Z',
                updated_at: '2024-01-01T00:00:00Z',
            }
            vi.mocked(api.get).mockResolvedValue({ data: [rule] })
            vi.mocked(api.put).mockResolvedValue({ data: { ...rule, enabled: false } })

            const store = useAlertsStore()
            await store.fetchRules()
            const result = await store.toggleRule('1', false)

            expect(result).toBe(true)
            expect(api.put).toHaveBeenCalledWith('/alerts/1', { enabled: false })
        })
    })

    describe('acknowledgeAlert', () => {
        it('acknowledges alert and updates status', async () => {
            const alert = {
                id: 'alert1',
                rule_id: 'rule1',
                rule_name: 'Test',
                alert_type: 'threshold',
                metric: 'error_rate',
                scope_type: 'all',
                triggered_at: '2024-01-01T00:00:00Z',
                trigger_value: 0.15,
                threshold: 0.1,
                status: 'triggered' as const,
            }
            vi.mocked(api.get).mockResolvedValue({ data: [alert] })
            vi.mocked(api.post).mockResolvedValue({})

            const store = useAlertsStore()
            await store.fetchActiveAlerts()
            const result = await store.acknowledgeAlert('alert1')

            expect(result).toBe(true)
            expect(api.post).toHaveBeenCalledWith('/alerts/alert1/acknowledge')
            expect(store.activeAlerts[0].status).toBe('acknowledged')
        })

        it('returns false on error', async () => {
            vi.mocked(api.post).mockRejectedValue(new Error('Failed'))

            const store = useAlertsStore()
            const result = await store.acknowledgeAlert('alert1')

            expect(result).toBe(false)
        })
    })
})
