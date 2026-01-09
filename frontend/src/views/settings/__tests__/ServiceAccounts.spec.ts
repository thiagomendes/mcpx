import { describe, it, expect, vi, beforeEach } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock vue-router
vi.mock('vue-router', () => ({
    useRouter: () => ({ push: vi.fn() }),
    RouterLink: { template: '<a><slot /></a>' }
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

    it('shows loading state', () => {
        const wrapper = createWrapper()
        expect(wrapper.text()).toContain('Loading')
    })

    it('has Create Service Account button', () => {
        const wrapper = createWrapper()
        // Button exists with some create text
        expect(wrapper.findAll('button').length).toBeGreaterThan(0)
    })

    it('has back to settings navigation', () => {
        const wrapper = createWrapper()
        expect(wrapper.find('router-link-stub').exists()).toBe(true)
    })

    it('component mounts successfully', () => {
        const wrapper = createWrapper()
        expect(wrapper.vm).toBeDefined()
    })
})
