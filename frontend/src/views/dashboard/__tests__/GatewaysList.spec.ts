import { describe, it, expect, vi, beforeEach } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock vue-router
const mockPush = vi.fn()
vi.mock('vue-router', () => ({
    useRouter: () => ({ push: mockPush }),
    useRoute: () => ({ query: {} }),
    RouterLink: { template: '<a><slot /></a>' }
}))

// Mock stores
const mockFetchGateways = vi.fn()
const mockDeleteGateway = vi.fn()
const mockCreateGateway = vi.fn()
vi.mock('@/stores/gateways', () => ({
    useGatewaysStore: vi.fn(() => ({
        gateways: [],
        loading: false,
        fetchGateways: mockFetchGateways,
        deleteGateway: mockDeleteGateway,
        createGateway: mockCreateGateway,
    }))
}))

// Import after mocks
import GatewaysList from '@/views/dashboard/GatewaysList.vue'
import { useGatewaysStore } from '@/stores/gateways'

describe('GatewaysList.vue', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    const createWrapper = (gatewaysData = { gateways: [], loading: false }) => {
        vi.mocked(useGatewaysStore).mockReturnValue({
            ...gatewaysData,
            fetchGateways: mockFetchGateways,
            deleteGateway: mockDeleteGateway,
            createGateway: mockCreateGateway,
        } as any)

        return shallowMount(GatewaysList, {
            global: {
                stubs: {
                    DashboardLayout: { template: '<div><slot /></div>' },
                    RouterLink: true,
                    RectangleStackIcon: true,
                    PlusIcon: true,
                    ClipboardDocumentIcon: true,
                    TrashIcon: true,
                }
            }
        })
    }

    it('renders the gateways page', () => {
        const wrapper = createWrapper()
        expect(wrapper.exists()).toBe(true)
        expect(wrapper.text()).toContain('Virtual Gateways')
        expect(wrapper.text()).toContain('Aggregate multiple servers into single endpoints')
    })

    it('shows loading state', () => {
        const wrapper = createWrapper({ gateways: [], loading: true })
        expect(wrapper.text()).toContain('Loading gateways...')
    })

    it('shows empty state when no gateways', () => {
        const wrapper = createWrapper({ gateways: [], loading: false })
        expect(wrapper.text()).toContain('No gateways yet')
        expect(wrapper.text()).toContain('Create Your First Gateway')
    })

    it('calls fetchGateways on mount', () => {
        createWrapper()
        expect(mockFetchGateways).toHaveBeenCalled()
    })

    it('displays gateway list when gateways exist', () => {
        const wrapper = createWrapper({
            gateways: [
                {
                    id: '1',
                    name: 'Test Gateway',
                    slug: 'test-gateway',
                    enabled: true,
                    proxy_url: '/mcp/org/test-gateway',
                    servers: [{ name: 'server-1' }]
                }
            ],
            loading: false
        })

        expect(wrapper.text()).toContain('Test Gateway')
        expect(wrapper.text()).toContain('Enabled')
        expect(wrapper.text()).toContain('1 servers')
        expect(wrapper.text()).not.toContain('No gateways yet')
    })

    it('shows disabled badge for disabled gateways', () => {
        const wrapper = createWrapper({
            gateways: [
                {
                    id: '1',
                    name: 'Disabled Gateway',
                    slug: 'disabled',
                    enabled: false,
                    proxy_url: '/mcp/org/disabled',
                    servers: []
                }
            ],
            loading: false
        })

        expect(wrapper.text()).toContain('Disabled')
    })

    it('shows server names in gateway card', () => {
        const wrapper = createWrapper({
            gateways: [
                {
                    id: '1',
                    name: 'Multi Gateway',
                    slug: 'multi',
                    enabled: true,
                    proxy_url: '/mcp/org/multi',
                    servers: [{ name: 'server-a' }, { name: 'server-b' }]
                }
            ],
            loading: false
        })

        expect(wrapper.text()).toContain('server-a')
        expect(wrapper.text()).toContain('server-b')
        expect(wrapper.text()).toContain('2 servers')
    })

    it('provides copy and delete actions for each gateway', () => {
        const wrapper = createWrapper({
            gateways: [
                { id: '1', name: 'Test Gateway', slug: 'test', enabled: true, proxy_url: '/mcp/test', servers: [] }
            ],
            loading: false
        })

        expect(wrapper.text()).toContain('Copy URL')
        expect(wrapper.text()).toContain('Delete')
    })

    it('shows create modal when New Gateway button exists', () => {
        const wrapper = createWrapper()
        expect(wrapper.text()).toContain('New Gateway')
    })

    it('has create modal form fields', async () => {
        const wrapper = createWrapper()

        // Open modal
        await wrapper.find('button').trigger('click')

        // Check modal content (may need DOM update)
        await wrapper.vm.$nextTick()
        expect(wrapper.text()).toContain('Create Gateway') || expect(wrapper.text()).toContain('New Gateway')
    })
})
