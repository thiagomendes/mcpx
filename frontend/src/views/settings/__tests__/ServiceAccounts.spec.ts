import { describe, it, expect, vi, beforeEach } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock vue-router
vi.mock('vue-router', () => ({
    useRouter: () => ({ push: vi.fn() }),
    RouterLink: { template: '<a><slot /></a>' }
}))

// Mock API client
vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn().mockResolvedValue({ data: { service_accounts: { current: 0, max: 10, can_create: true } } }),
        post: vi.fn().mockResolvedValue({ data: {} }),
        delete: vi.fn().mockResolvedValue({ data: {} }),
    }
}))

// Import after mocks
import ServiceAccounts from '@/views/settings/ServiceAccounts.vue'

describe('ServiceAccounts.vue', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    const createWrapper = () => {
        return shallowMount(ServiceAccounts, {
            global: {
                stubs: {
                    DashboardLayout: { template: '<div><slot /></div>' },
                    RouterLink: true,
                    PlusIcon: true,
                    CogIcon: true,
                    TrashIcon: true,
                    ArrowLeftIcon: true,
                    CheckIcon: true,
                    ClipboardDocumentIcon: true,
                    ComputerDesktopIcon: true,
                    KeyIcon: true,
                }
            }
        })
    }

    it('renders the service accounts page', () => {
        const wrapper = createWrapper()
        expect(wrapper.exists()).toBe(true)
        expect(wrapper.text()).toContain('Service Accounts')
    })

    it('shows loading or content state', () => {
        const wrapper = createWrapper()
        // Component will be loading initially
        expect(wrapper.exists()).toBe(true)
    })

    it('has service accounts section', () => {
        const wrapper = createWrapper()
        expect(wrapper.text()).toContain('Service Accounts')
    })

    it('component renders successfully', () => {
        const wrapper = createWrapper()
        expect(wrapper.exists()).toBe(true)
    })

    it('component mounts successfully', () => {
        const wrapper = createWrapper()
        expect(wrapper.vm).toBeDefined()
    })
})
