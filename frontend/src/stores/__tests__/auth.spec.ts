import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useAuthStore } from '@/stores/auth'
import api from '@/api/client'

vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn(),
        post: vi.fn(),
    },
}))

// Mock localStorage
const localStorageMock = {
    store: {} as Record<string, string>,
    getItem: vi.fn((key: string) => localStorageMock.store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
        localStorageMock.store[key] = value
    }),
    removeItem: vi.fn((key: string) => {
        delete localStorageMock.store[key]
    }),
    clear: vi.fn(() => {
        localStorageMock.store = {}
    }),
}
Object.defineProperty(window, 'localStorage', { value: localStorageMock })

describe('Auth Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
        localStorageMock.clear()
    })

    afterEach(() => {
        vi.restoreAllMocks()
    })

    describe('initial state', () => {
        it('starts with null values when no token in localStorage', () => {
            const store = useAuthStore()
            expect(store.user).toBeNull()
            expect(store.orgs).toEqual([])
            expect(store.isAuthenticated).toBe(false)
        })

        it('reads token from localStorage on init', () => {
            localStorageMock.store['token'] = 'test-token'
            const store = useAuthStore()
            expect(store.token).toBe('test-token')
        })
    })

    describe('setToken', () => {
        it('sets token and saves to localStorage', () => {
            const store = useAuthStore()
            store.setToken('new-token')
            expect(store.token).toBe('new-token')
            expect(localStorageMock.setItem).toHaveBeenCalledWith('token', 'new-token')
        })
    })

    describe('fetchUser', () => {
        it('returns null when no token', async () => {
            const store = useAuthStore()
            const result = await store.fetchUser()
            expect(result).toBeNull()
            expect(api.get).not.toHaveBeenCalled()
        })

        it('fetches user data and updates state', async () => {
            localStorageMock.store['token'] = 'test-token'
            const mockResponse = {
                data: {
                    user: { id: '1', email: 'test@example.com', name: 'Test User', avatar_url: null },
                    orgs: [{ id: 'org1', name: 'Test Org', slug: 'test-org', is_personal: false, role: 'owner', created_at: '2024-01-01' }],
                    current_org_id: 'org1',
                    identities: [{ provider: 'google', created_at: '2024-01-01' }],
                    auth_provider: 'google',
                },
            }
            vi.mocked(api.get).mockResolvedValue(mockResponse)

            const store = useAuthStore()
            const result = await store.fetchUser()

            expect(api.get).toHaveBeenCalledWith('/auth/me')
            expect(result).toEqual(mockResponse.data.user)
            expect(store.user).toEqual(mockResponse.data.user)
            expect(store.orgs).toEqual(mockResponse.data.orgs)
            expect(store.currentOrgId).toBe('org1')
        })

        it('logs out on fetch error', async () => {
            localStorageMock.store['token'] = 'test-token'
            vi.mocked(api.get).mockRejectedValue(new Error('Network error'))

            const store = useAuthStore()
            const result = await store.fetchUser()

            expect(result).toBeNull()
            expect(store.token).toBeNull()
            expect(store.user).toBeNull()
        })
    })

    describe('setCurrentOrg', () => {
        it('sets current org id and saves to localStorage', () => {
            const store = useAuthStore()
            store.setCurrentOrg('org123')
            expect(store.currentOrgId).toBe('org123')
            expect(localStorageMock.setItem).toHaveBeenCalledWith('currentOrgId', 'org123')
        })
    })

    describe('switchOrg', () => {
        it('switches org and updates token', async () => {
            localStorageMock.store['token'] = 'old-token'
            const mockResponse = {
                data: {
                    token: 'new-token',
                    org: { id: 'org2', name: 'Other Org', slug: 'other-org', is_personal: false, role: 'admin', created_at: '2024-01-01' },
                },
            }
            vi.mocked(api.post).mockResolvedValue(mockResponse)

            const store = useAuthStore()
            store.orgs = [
                { id: 'org1', name: 'Test Org', slug: 'test-org', is_personal: false, role: 'owner', created_at: '2024-01-01' },
                { id: 'org2', name: 'Other Org', slug: 'other-org', is_personal: false, role: 'admin', created_at: '2024-01-01' },
            ]

            const result = await store.switchOrg('org2')

            expect(result).toBe(true)
            expect(api.post).toHaveBeenCalledWith('/auth/switch-org/org2')
            expect(store.token).toBe('new-token')
            expect(store.currentOrgId).toBe('org2')
        })

        it('returns false on switch error', async () => {
            localStorageMock.store['token'] = 'test-token'
            vi.mocked(api.post).mockRejectedValue(new Error('Switch failed'))

            const store = useAuthStore()
            const result = await store.switchOrg('org2')

            expect(result).toBe(false)
        })
    })

    describe('logout', () => {
        it('clears all state and localStorage', () => {
            const store = useAuthStore()
            store.token = 'test-token'
            store.user = { id: '1', email: 'test@example.com', name: 'Test', avatar_url: null }
            store.orgs = [{ id: 'org1', name: 'Org', slug: 'org', is_personal: false, role: 'owner', created_at: '2024-01-01' }]

            store.logout()

            expect(store.token).toBeNull()
            expect(store.user).toBeNull()
            expect(store.orgs).toEqual([])
            expect(store.currentOrgId).toBeNull()
            expect(localStorageMock.removeItem).toHaveBeenCalledWith('token')
            expect(localStorageMock.removeItem).toHaveBeenCalledWith('currentOrgId')
        })
    })

    describe('computed properties', () => {
        it('isAuthenticated returns true when token exists', () => {
            localStorageMock.store['token'] = 'test-token'
            const store = useAuthStore()
            expect(store.isAuthenticated).toBe(true)
        })

        it('currentOrg returns matching org by currentOrgId', () => {
            const store = useAuthStore()
            store.orgs = [
                { id: 'org1', name: 'Org 1', slug: 'org1', is_personal: false, role: 'owner', created_at: '2024-01-01' },
                { id: 'org2', name: 'Org 2', slug: 'org2', is_personal: true, role: 'member', created_at: '2024-01-01' },
            ]
            store.setCurrentOrg('org2')

            expect(store.currentOrg?.id).toBe('org2')
            expect(store.orgSlug).toBe('org2')
            expect(store.isPersonalOrg).toBe(true)
            expect(store.userRole).toBe('member')
        })

        it('canAdmin returns true for owner or admin', () => {
            const store = useAuthStore()
            store.orgs = [{ id: 'org1', name: 'Org', slug: 'org', is_personal: false, role: 'owner', created_at: '2024-01-01' }]
            store.setCurrentOrg('org1')
            expect(store.canAdmin).toBe(true)

            store.orgs = [{ id: 'org1', name: 'Org', slug: 'org', is_personal: false, role: 'admin', created_at: '2024-01-01' }]
            expect(store.canAdmin).toBe(true)

            store.orgs = [{ id: 'org1', name: 'Org', slug: 'org', is_personal: false, role: 'member', created_at: '2024-01-01' }]
            expect(store.canAdmin).toBe(false)
        })
    })

    describe('OAuth login helpers', () => {
        it('loginWithGoogle redirects to Google auth endpoint', () => {
            // Mock window.location using Object.defineProperty
            Object.defineProperty(window, 'location', {
                value: { href: '' },
                writable: true,
                configurable: true
            })

            const store = useAuthStore()
            store.loginWithGoogle()

            expect(window.location.href).toContain('/auth/google')
        })

        it('loginWithGithub redirects to GitHub auth endpoint', () => {
            Object.defineProperty(window, 'location', {
                value: { href: '' },
                writable: true,
                configurable: true
            })

            const store = useAuthStore()
            store.loginWithGithub()

            expect(window.location.href).toContain('/auth/github')
        })

        it('loginWithMicrosoft redirects to Microsoft auth endpoint', () => {
            Object.defineProperty(window, 'location', {
                value: { href: '' },
                writable: true,
                configurable: true
            })

            const store = useAuthStore()
            store.loginWithMicrosoft()

            expect(window.location.href).toContain('/auth/microsoft')
        })
    })
})
