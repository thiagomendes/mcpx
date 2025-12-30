import { describe, it, expect, vi, beforeEach } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock vue-router
const mockParams = { name: 'test-server' }
vi.mock('vue-router', () => ({
    useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
    useRoute: () => ({ params: mockParams }),
    RouterLink: { template: '<a><slot /></a>' }
}))

// Mock stores
const mockServer = {
    name: 'test-server',
    url: 'http://localhost:3000',
    status: 'healthy',
    transport: 'http',
    enabled: true,
    auth_type: 'none',
    auth: { oauth: null, api_key: null, bearer_token: null }
}
vi.mock('@/stores/servers', () => ({
    useServersStore: vi.fn(() => ({
        servers: [mockServer],
        getServerByName: vi.fn(() => mockServer),
        fetchServer: vi.fn().mockResolvedValue({}),
        fetchServers: vi.fn().mockResolvedValue([]),
        updateServer: vi.fn().mockResolvedValue({}),
        deleteServer: vi.fn().mockResolvedValue({}),
    }))
}))

// Import after mocks
import ServerDetails from '@/views/dashboard/ServerDetails.vue'

describe('ServerDetails.vue', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    const createWrapper = () => {
        return shallowMount(ServerDetails, {
            global: {
                stubs: {
                    DashboardLayout: { template: '<div><slot /></div>' },
                    RouterLink: true,
                    ServerIcon: true,
                    ArrowLeftIcon: true,
                    TrashIcon: true,
                    PencilIcon: true,
                    ClipboardDocumentIcon: true,
                    CheckIcon: true,
                    XMarkIcon: true,
                    PlayIcon: true,
                    StopIcon: true,
                    KeyIcon: true,
                    EyeIcon: true,
                    EyeSlashIcon: true,
                }
            }
        })
    }

    it('renders the server details page', () => {
        const wrapper = createWrapper()
        expect(wrapper.exists()).toBe(true)
    })

    it('component mounts successfully', () => {
        const wrapper = createWrapper()
        expect(wrapper.vm).toBeDefined()
    })
})
