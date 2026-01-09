import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import api from '@/api/client'

// ============================================
// TYPES
// ============================================

interface User {
    id: string
    email: string
    name: string | null
    avatar_url: string | null
}

interface Organization {
    id: string
    name: string
    slug: string
    is_personal: boolean
    role: string
    created_at: string
}

interface Identity {
    provider: string
    created_at: string
}

interface MeResponse {
    user: User
    orgs: Organization[]
    current_org_id: string
    identities: Identity[]
    auth_provider: string
}

// ============================================
// AUTH STORE
// ============================================

export const useAuthStore = defineStore('auth', () => {
    // State
    const token = ref<string | null>(localStorage.getItem('token'))
    const user = ref<User | null>(null)
    const orgs = ref<Organization[]>([])
    const currentOrgId = ref<string | null>(localStorage.getItem('currentOrgId'))
    const identities = ref<Identity[]>([])
    const authProvider = ref<string | null>(null)

    // Computed
    const isAuthenticated = computed(() => !!token.value)

    const currentOrg = computed(() =>
        orgs.value.find(o => o.id === currentOrgId.value) || orgs.value[0] || null
    )

    const orgSlug = computed(() => currentOrg.value?.slug || '')

    const isPersonalOrg = computed(() => currentOrg.value?.is_personal || false)

    const userRole = computed(() => currentOrg.value?.role || 'member')

    const canAdmin = computed(() => ['owner', 'admin'].includes(userRole.value))

    // Actions
    function setToken(newToken: string) {
        token.value = newToken
        localStorage.setItem('token', newToken)
    }

    async function fetchUser(): Promise<User | null> {
        if (!token.value) return null

        try {
            const response = await api.get<MeResponse>('/auth/me')
            user.value = response.data.user
            orgs.value = response.data.orgs
            identities.value = response.data.identities
            authProvider.value = response.data.auth_provider

            // Set current org if not already set
            if (!currentOrgId.value && response.data.current_org_id) {
                setCurrentOrg(response.data.current_org_id)
            }

            return user.value
        } catch (error) {
            console.error('Failed to fetch user:', error)
            logout()
            return null
        }
    }

    function setCurrentOrg(orgId: string) {
        currentOrgId.value = orgId
        localStorage.setItem('currentOrgId', orgId)
    }

    async function switchOrg(orgId: string): Promise<boolean> {
        try {
            const response = await api.post<{ token: string; org: Organization }>(`/auth/switch-org/${orgId}`)
            setToken(response.data.token)
            setCurrentOrg(orgId)

            // Update the org in the list
            const idx = orgs.value.findIndex(o => o.id === orgId)
            if (idx >= 0) {
                orgs.value[idx] = response.data.org
            }

            return true
        } catch (error) {
            console.error('Failed to switch org:', error)
            return false
        }
    }

    function logout() {
        token.value = null
        user.value = null
        orgs.value = []
        currentOrgId.value = null
        identities.value = []
        authProvider.value = null
        localStorage.removeItem('token')
        localStorage.removeItem('currentOrgId')
    }

    // OAuth login helpers
    function loginWithGoogle() {
        window.location.href = `${import.meta.env.VITE_API_URL}/auth/google`
    }

    function loginWithGithub() {
        window.location.href = `${import.meta.env.VITE_API_URL}/auth/github`
    }

    function loginWithMicrosoft() {
        window.location.href = `${import.meta.env.VITE_API_URL}/auth/microsoft`
    }

    return {
        // State
        token,
        user,
        orgs,
        currentOrgId,
        identities,
        authProvider,

        // Computed
        isAuthenticated,
        currentOrg,
        orgSlug,
        isPersonalOrg,
        userRole,
        canAdmin,

        // Actions
        setToken,
        fetchUser,
        setCurrentOrg,
        switchOrg,
        logout,
        loginWithGoogle,
        loginWithGithub,
        loginWithMicrosoft,
    }
})
