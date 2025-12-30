import { describe, it, expect, vi, beforeEach } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock vue-router
const mockParams = { slug: 'test-gateway' }
vi.mock('vue-router', () => ({
    useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
    useRoute: () => ({ params: mockParams }),
    RouterLink: { template: '<a><slot /></a>' }
}))

// Mock stores
vi.mock('@/stores/gateways', () => ({
    useGatewaysStore: vi.fn(() => ({
        gateways: [{ slug: 'test-gateway', name: 'Test Gateway', enabled: true, servers: [] }],
        getGatewayBySlug: vi.fn(() => ({ slug: 'test-gateway', name: 'Test Gateway', enabled: true, servers: [] })),
        fetchGateway: vi.fn().mockResolvedValue({}),
        fetchGateways: vi.fn().mockResolvedValue([]),
        updateGateway: vi.fn().mockResolvedValue({}),
        deleteGateway: vi.fn().mockResolvedValue({}),
        addServerToGateway: vi.fn().mockResolvedValue({}),
        removeServerFromGateway: vi.fn().mockResolvedValue({}),
    }))
}))

vi.mock('@/stores/servers', () => ({
    useServersStore: vi.fn(() => ({
        servers: [],
        fetchServers: vi.fn().mockResolvedValue([]),
    }))
}))

// Import after mocks
import GatewayDetails from '@/views/dashboard/GatewayDetails.vue'

describe('GatewayDetails.vue', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    const createWrapper = () => {
        return shallowMount(GatewayDetails, {
            global: {
                stubs: {
                    DashboardLayout: { template: '<div><slot /></div>' },
                    RouterLink: true,
                    RectangleStackIcon: true,
                    ArrowLeftIcon: true,
                    TrashIcon: true,
                    PlusIcon: true,
                    ClipboardDocumentIcon: true,
                    CheckIcon: true,
                    XMarkIcon: true,
                }
            }
        })
    }

    it('renders the gateway details page', () => {
        const wrapper = createWrapper()
        expect(wrapper.exists()).toBe(true)
    })

    it('component mounts successfully', () => {
        const wrapper = createWrapper()
        expect(wrapper.vm).toBeDefined()
    })
})
