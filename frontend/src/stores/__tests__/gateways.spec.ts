import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useGatewaysStore } from '@/stores/gateways'
import api from '@/api/client'

vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn(),
        post: vi.fn(),
        put: vi.fn(),
        delete: vi.fn(),
    },
}))

describe('Gateways Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    describe('fetchGateways', () => {
        it('fetches gateways and updates state', async () => {
            const mockGateways = [
                {
                    id: '1',
                    name: 'Dev Gateway',
                    slug: 'dev',
                    enabled: true,
                    proxy_url: 'http://localhost:8080/mcp/user1/dev',
                    servers: [{ name: 'cloudflare', priority: 0 }],
                    created_at: '2024-01-01T00:00:00Z',
                    updated_at: '2024-01-01T00:00:00Z',
                },
            ]
            vi.mocked(api.get).mockResolvedValue({ data: mockGateways })

            const store = useGatewaysStore()
            await store.fetchGateways()

            expect(api.get).toHaveBeenCalledWith('/gateways')
            expect(store.gateways).toEqual(mockGateways)
            expect(store.loading).toBe(false)
            expect(store.error).toBeNull()
        })

        it('handles fetch error', async () => {
            vi.mocked(api.get).mockRejectedValue(new Error('Network error'))

            const store = useGatewaysStore()
            await store.fetchGateways()

            expect(store.error).toBe('Failed to fetch gateways')
            expect(store.loading).toBe(false)
        })
    })

    describe('createGateway', () => {
        it('creates gateway and adds to state', async () => {
            const newGateway = {
                id: '2',
                name: 'Prod Gateway',
                slug: 'prod',
                enabled: true,
                proxy_url: 'http://localhost:8080/mcp/user1/prod',
                servers: [],
                created_at: '2024-01-02T00:00:00Z',
                updated_at: '2024-01-02T00:00:00Z',
            }
            vi.mocked(api.post).mockResolvedValue({ data: newGateway })

            const store = useGatewaysStore()
            const result = await store.createGateway({ name: 'Prod Gateway', slug: 'prod' })

            expect(api.post).toHaveBeenCalledWith('/gateways', { name: 'Prod Gateway', slug: 'prod' })
            expect(result).toEqual(newGateway)
            expect(store.gateways[0]).toEqual(newGateway)
        })
    })

    describe('updateGateway', () => {
        it('updates gateway in state', async () => {
            const existingGateway = {
                id: '1',
                name: 'Dev Gateway',
                slug: 'dev',
                enabled: true,
                proxy_url: 'http://localhost:8080/mcp/user1/dev',
                servers: [],
                created_at: '2024-01-01T00:00:00Z',
                updated_at: '2024-01-01T00:00:00Z',
            }
            const updatedGateway = { ...existingGateway, enabled: false }

            vi.mocked(api.get).mockResolvedValue({ data: [existingGateway] })
            vi.mocked(api.put).mockResolvedValue({ data: updatedGateway })

            const store = useGatewaysStore()
            await store.fetchGateways()
            const result = await store.updateGateway('dev', { enabled: false })

            expect(api.put).toHaveBeenCalledWith('/gateways/dev', { enabled: false })
            expect(result.enabled).toBe(false)
            expect(store.gateways[0].enabled).toBe(false)
        })
    })

    describe('deleteGateway', () => {
        it('removes gateway from state', async () => {
            const gateway = {
                id: '1',
                name: 'Dev Gateway',
                slug: 'dev',
                enabled: true,
                proxy_url: 'http://localhost:8080/mcp/user1/dev',
                servers: [],
                created_at: '2024-01-01T00:00:00Z',
                updated_at: '2024-01-01T00:00:00Z',
            }
            vi.mocked(api.get).mockResolvedValue({ data: [gateway] })
            vi.mocked(api.delete).mockResolvedValue({})

            const store = useGatewaysStore()
            await store.fetchGateways()
            expect(store.gateways.length).toBe(1)

            await store.deleteGateway('dev')

            expect(api.delete).toHaveBeenCalledWith('/gateways/dev')
            expect(store.gateways.length).toBe(0)
        })
    })

    describe('addServerToGateway', () => {
        it('adds server to gateway', async () => {
            vi.mocked(api.post).mockResolvedValue({})
            vi.mocked(api.get).mockResolvedValue({ data: [] })

            const store = useGatewaysStore()
            await store.addServerToGateway('dev', 'cloudflare')

            expect(api.post).toHaveBeenCalledWith('/gateways/dev/servers', { server_name: 'cloudflare' })
            expect(api.get).toHaveBeenCalledWith('/gateways')
        })
    })

    describe('removeServerFromGateway', () => {
        it('removes server from gateway', async () => {
            vi.mocked(api.delete).mockResolvedValue({})
            vi.mocked(api.get).mockResolvedValue({ data: [] })

            const store = useGatewaysStore()
            await store.removeServerFromGateway('dev', 'cloudflare')

            expect(api.delete).toHaveBeenCalledWith('/gateways/dev/servers/cloudflare')
            expect(api.get).toHaveBeenCalledWith('/gateways')
        })
    })

    describe('getGatewayBySlug', () => {
        it('returns gateway by slug', async () => {
            const gateway = {
                id: '1',
                name: 'Dev Gateway',
                slug: 'dev',
                enabled: true,
                proxy_url: 'http://localhost:8080/mcp/user1/dev',
                servers: [],
                created_at: '2024-01-01T00:00:00Z',
                updated_at: '2024-01-01T00:00:00Z',
            }
            vi.mocked(api.get).mockResolvedValue({ data: [gateway] })

            const store = useGatewaysStore()
            await store.fetchGateways()

            expect(store.getGatewayBySlug('dev')).toEqual(gateway)
            expect(store.getGatewayBySlug('nonexistent')).toBeUndefined()
        })
    })
})
