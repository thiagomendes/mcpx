import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock vue-router
vi.mock('vue-router', () => ({
    useRouter: () => ({
        push: vi.fn(),
    }),
    RouterLink: {
        template: '<a><slot /></a>'
    }
}))

// Mock identity providers
vi.mock('@/lib/identityProviders', () => ({
    getEnabledProviders: vi.fn(() => [
        { id: 'google', name: 'Google', color: '#4285f4' },
        { id: 'github', name: 'GitHub', color: '#333' },
    ]),
    loginWithProvider: vi.fn(),
}))

// Import after mocks
import SignIn from '@/views/SignIn.vue'
import { loginWithProvider, getEnabledProviders } from '@/lib/identityProviders'

describe('SignIn.vue', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    it('renders the sign in page', () => {
        const wrapper = shallowMount(SignIn, {
            global: {
                stubs: {
                    RouterLink: true,
                    BoltIcon: true,
                    ArrowLeftIcon: true,
                    GoogleIcon: true,
                    GitHubIcon: true,
                    MicrosoftIcon: true,
                }
            }
        })

        expect(wrapper.exists()).toBe(true)
        expect(wrapper.text()).toContain('Welcome back')
        expect(wrapper.text()).toContain('Sign in to manage your MCP servers')
    })

    it('displays enabled identity providers', () => {
        const wrapper = shallowMount(SignIn, {
            global: {
                stubs: {
                    RouterLink: true,
                    BoltIcon: true,
                    ArrowLeftIcon: true,
                    GoogleIcon: true,
                    GitHubIcon: true,
                    MicrosoftIcon: true,
                }
            }
        })

        // Should show buttons for each enabled provider
        expect(wrapper.text()).toContain('Sign in with Google')
        expect(wrapper.text()).toContain('Sign in with GitHub')
    })

    it('calls loginWithProvider when button is clicked', async () => {
        const wrapper = shallowMount(SignIn, {
            global: {
                stubs: {
                    RouterLink: true,
                    BoltIcon: true,
                    ArrowLeftIcon: true,
                    GoogleIcon: true,
                    GitHubIcon: true,
                    MicrosoftIcon: true,
                }
            }
        })

        const buttons = wrapper.findAll('button')
        expect(buttons.length).toBeGreaterThan(0)

        await buttons[0].trigger('click')
        expect(loginWithProvider).toHaveBeenCalledWith('google')
    })

    it('has back to home navigation', () => {
        const wrapper = shallowMount(SignIn, {
            global: {
                stubs: {
                    RouterLink: true,
                    BoltIcon: true,
                    ArrowLeftIcon: true,
                    GoogleIcon: true,
                    GitHubIcon: true,
                    MicrosoftIcon: true,
                }
            }
        })

        // RouterLink is stubbed - verify it exists
        expect(wrapper.find('router-link-stub').exists()).toBe(true)
    })

    it('shows terms of service notice', () => {
        const wrapper = shallowMount(SignIn, {
            global: {
                stubs: {
                    RouterLink: true,
                    BoltIcon: true,
                    ArrowLeftIcon: true,
                    GoogleIcon: true,
                    GitHubIcon: true,
                    MicrosoftIcon: true,
                }
            }
        })

        expect(wrapper.text()).toContain('Terms of Service')
    })
})
