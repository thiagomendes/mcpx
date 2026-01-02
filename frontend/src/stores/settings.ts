import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Setting {
    key: string
    value: string
    value_type: string
    description: string | null
}

export interface SettingsResponse {
    settings: Setting[]
    can_edit: boolean
}

// Setting metadata for UI
export interface SettingMeta {
    key: string
    label: string
    category: 'tokens' | 'limits'
    unit: string
    min: number
    max: number
    warning?: string
}

export const SETTING_METADATA: SettingMeta[] = [
    // Tokens (per-org configurable)
    { key: 'jwt_expiry_days', label: 'User Session Duration', category: 'tokens', unit: 'days', min: 1, max: 90 },
    { key: 'm2m_token_expiry_hours', label: 'Service Account Token Duration', category: 'tokens', unit: 'hours', min: 1, max: 24 },

    // Limits (per-org configurable)
    { key: 'max_servers_per_org', label: 'Max Servers per Org', category: 'limits', unit: '', min: 1, max: 1000 },
    { key: 'max_gateways_per_org', label: 'Max Gateways per Org', category: 'limits', unit: '', min: 1, max: 500 },
    { key: 'max_pats_per_user', label: 'Max PATs per User', category: 'limits', unit: '', min: 1, max: 100 },
    { key: 'max_service_accounts_per_org', label: 'Max Service Accounts per Org', category: 'limits', unit: '', min: 1, max: 100 },
]

export const useSettingsStore = defineStore('settings', () => {
    const settings = ref<Setting[]>([])
    const canEdit = ref(false)
    const loading = ref(false)
    const error = ref<string | null>(null)
    const saving = ref<string | null>(null) // key being saved

    async function fetchSettings() {
        loading.value = true
        error.value = null
        try {
            const response = await api.get<SettingsResponse>('/settings')
            settings.value = response.data.settings
            canEdit.value = response.data.can_edit
        } catch (e) {
            error.value = e instanceof Error ? e.message : 'Failed to fetch settings'
            console.error('Error fetching settings:', e)
        } finally {
            loading.value = false
        }
    }

    async function updateSetting(key: string, value: string): Promise<boolean> {
        saving.value = key
        error.value = null
        try {
            await api.put(`/settings/${key}`, { value })
            // Update local state
            const setting = settings.value.find(s => s.key === key)
            if (setting) {
                setting.value = value
            }
            return true
        } catch (e: unknown) {
            const errorMessage = (e as { response?: { data?: string } })?.response?.data ||
                (e instanceof Error ? e.message : 'Failed to update setting')
            error.value = errorMessage
            console.error('Error updating setting:', e)
            return false
        } finally {
            saving.value = null
        }
    }

    function getSettingValue(key: string): string {
        const setting = settings.value.find(s => s.key === key)
        return setting?.value || ''
    }

    function getSettingMeta(key: string): SettingMeta | undefined {
        return SETTING_METADATA.find(m => m.key === key)
    }

    function getSettingsByCategory(category: string): Setting[] {
        const categoryKeys = SETTING_METADATA.filter(m => m.category === category).map(m => m.key)
        return settings.value.filter(s => categoryKeys.includes(s.key))
    }

    return {
        settings,
        canEdit,
        loading,
        error,
        saving,
        fetchSettings,
        updateSetting,
        getSettingValue,
        getSettingMeta,
        getSettingsByCategory,
    }
})
