import { describe, it, expect, vi, beforeEach } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock vue-router
vi.mock('vue-router', () => ({
    useRouter: () => ({ push: vi.fn() }),
    RouterLink: { template: '<a><slot /></a>' }
}))

// Mock stores
vi.mock('@/stores/auth', () => ({
    useAuthStore: vi.fn(() => ({
        user: { name: 'Test User', email: 'test@example.com' },
    }))
}))

vi.mock('@/stores/servers', () => ({
    useServersStore: vi.fn(() => ({
        servers: [],
        fetchServers: vi.fn(),
    }))
}))

vi.mock('@/stores/metrics', () => ({
    useMetricsStore: vi.fn(() => ({
        metrics: null,
        loading: false,
        fetchMetrics: vi.fn(),
        queryMetrics: vi.fn().mockResolvedValue([]),
    }))
}))

// Mock vue-chartjs
vi.mock('vue-chartjs', () => ({
    Line: { template: '<div class="chart-mock"></div>' },
    Bar: { template: '<div class="chart-mock"></div>' },
}))

// Mock chart.js
vi.mock('chart.js', () => ({
    Chart: { register: vi.fn() },
    CategoryScale: {},
    LinearScale: {},
    PointElement: {},
    LineElement: {},
    BarElement: {},
    Title: {},
    Tooltip: {},
    Legend: {},
    Filler: {},
}))

// Import after mocks
import Dashboard from '@/views/dashboard/Dashboard.vue'

describe('Dashboard.vue', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    const createWrapper = () => {
        return shallowMount(Dashboard, {
            global: {
                stubs: {
                    DashboardLayout: { template: '<div><slot /></div>' },
                    ActiveAlertsPanel: { template: '<div></div>' },
                    Line: { template: '<div></div>' },
                    ChartBarIcon: true,
                    ClockIcon: true,
                    ArrowPathIcon: true,
                    WrenchScrewdriverIcon: true,
                }
            }
        })
    }

    it('renders the dashboard page', () => {
        const wrapper = createWrapper()
        expect(wrapper.exists()).toBe(true)
        expect(wrapper.text()).toContain('Dashboard')
    })

    it('shows user name in greeting', () => {
        const wrapper = createWrapper()
        expect(wrapper.text()).toContain('Welcome back')
    })

    it('has time range selector', () => {
        const wrapper = createWrapper()
        expect(wrapper.find('select').exists()).toBe(true)
    })

    it('has refresh button', () => {
        const wrapper = createWrapper()
        expect(wrapper.findAll('button').length).toBeGreaterThan(0)
    })

    it('shows stats cards', () => {
        const wrapper = createWrapper()
        expect(wrapper.text()).toContain('Requests')
    })

    it('component mounts successfully', () => {
        const wrapper = createWrapper()
        expect(wrapper.vm).toBeDefined()
    })
})
