import { describe, it, expect, vi, beforeEach } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock vue-router
vi.mock('vue-router', () => ({
    useRouter: () => ({ push: vi.fn() }),
    RouterLink: { template: '<a><slot /></a>' }
}))

// Import after mocks
import Tokens from '@/views/settings/Tokens.vue'

describe('Tokens.vue', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    const createWrapper = () => {
        return shallowMount(Tokens, {
            global: {
                stubs: {
                    DashboardLayout: { template: '<div><slot /></div>' },
                    RouterLink: true,
                    PlusIcon: true,
                    KeyIcon: true,
                    TrashIcon: true,
                    ArrowLeftIcon: true,
                    CheckIcon: true,
                    ClipboardDocumentIcon: true,
                }
            }
        })
    }

    it('renders the tokens page', () => {
        const wrapper = createWrapper()
        expect(wrapper.exists()).toBe(true)
        expect(wrapper.text()).toContain('Personal Access Tokens')
    })

    it('shows loading state', () => {
        const wrapper = createWrapper()
        // Loading state shows initially
        expect(wrapper.text()).toContain('Loading tokens...')
    })

    it('has New Token button', () => {
        const wrapper = createWrapper()
        expect(wrapper.text()).toContain('New Token')
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
