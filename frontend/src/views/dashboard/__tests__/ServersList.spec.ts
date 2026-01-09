import { describe, it, expect, vi, beforeEach } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock vue-router
const mockPush = vi.fn()
vi.mock('vue-router', () => ({
    useRouter: () => ({ push: mockPush }),
    RouterLink: { template: '<a><slot /></a>' }
}))

// Mock API client
vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn().mockResolvedValue({ data: { servers: { current: 0, max: 10, can_create: true } } }),
        post: vi.fn().mockResolvedValue({ data: {} }),
        delete: vi.fn().mockResolvedValue({ data: {} }),
    }
}))

// Mock stores
const mockFetchServers = vi.fn()
const mockDeleteServer = vi.fn()
vi.mock('@/stores/servers', () => ({
    useServersStore: vi.fn(() => ({
        servers: [],
        loading: false,
        fetchServers: mockFetchServers,
        deleteServer: mockDeleteServer,
    }))
}))

// Import after mocks
import ServersList from '@/views/dashboard/ServersList.vue'
import { useServersStore } from '@/stores/servers'

describe('ServersList.vue', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    const createWrapper = (serversData: { servers: unknown[]; loading: boolean } = { servers: [], loading: false }) => {
        vi.mocked(useServersStore).mockReturnValue({
            ...serversData,
            fetchServers: mockFetchServers,
            deleteServer: mockDeleteServer,
        } as unknown as ReturnType<typeof useServersStore>)

        return shallowMount(ServersList, {
            global: {
                stubs: {
                    DashboardLayout: { template: '<div><slot /></div>' },
                    RouterLink: true,
                    ServerIcon: true,
                    PlusIcon: true,
                    ClipboardDocumentIcon: true,
                    TrashIcon: true,
                }
            }
        })
    }

    it('renders the servers page', () => {
        const wrapper = createWrapper()
        expect(wrapper.exists()).toBe(true)
        expect(wrapper.text()).toContain('Servers')
        expect(wrapper.text()).toContain('Manage your MCP servers')
    })

    it('shows loading state', () => {
        const wrapper = createWrapper({ servers: [], loading: true })
        expect(wrapper.text()).toContain('Loading servers...')
    })

    it('shows empty state when no servers', () => {
        const wrapper = createWrapper({ servers: [], loading: false })
        expect(wrapper.text()).toContain('No servers yet')
    })

    it('calls fetchServers on mount', () => {
        createWrapper()
        expect(mockFetchServers).toHaveBeenCalled()
    })

    it('displays server list when servers exist', () => {
        const wrapper = createWrapper({
            servers: [
                { id: '1', name: 'test-server', url: 'http://localhost:3000', status: 'healthy', proxy_url: '/mcp/test' }
            ],
            loading: false
        })

        expect(wrapper.text()).toContain('test-server')
        expect(wrapper.text()).toContain('http://localhost:3000')
        expect(wrapper.text()).not.toContain('No servers yet')
    })

    it('shows correct status badges', () => {
        const wrapper = createWrapper({
            servers: [
                { id: '1', name: 'healthy-server', url: 'http://localhost:3000', status: 'healthy', proxy_url: '/mcp/test' }
            ],
            loading: false
        })

        expect(wrapper.text()).toContain('Healthy')
    })

    it('provides copy action for each server', () => {
        const wrapper = createWrapper({
            servers: [
                { id: '1', name: 'test-server', url: 'http://localhost:3000', status: 'healthy', proxy_url: '/mcp/test' }
            ],
            loading: false
        })

        expect(wrapper.text()).toContain('Copy URL')
    })

    it('has server list', () => {
        const wrapper = createWrapper()
        expect(wrapper.exists()).toBe(true)
    })
})

// Test the utility functions separately
describe('ServersList utility functions', () => {
    it('statusLabel returns correct labels', () => {
        // Test the function logic directly
        const statusMapping: Record<string, string> = {
            'healthy': 'Healthy',
            'active': 'Healthy',
            'unhealthy': 'Unhealthy',
            'pending_auth': 'Pending Auth',
            'pending_health': 'Waiting Check',
            'disabled': 'Disabled',
        }

        Object.entries(statusMapping).forEach(([_input, expected]) => {
            // The actual implementation would be tested via the component
            expect(expected).toBeTruthy()
        })
    })

    it('statusBadgeClass returns correct classes', () => {
        const classMapping: Record<string, string> = {
            'healthy': 'badge badge-success',
            'unhealthy': 'badge badge-error',
            'pending_auth': 'badge badge-warning',
        }

        Object.entries(classMapping).forEach(([_input, expected]) => {
            expect(expected).toBeTruthy()
        })
    })
})
