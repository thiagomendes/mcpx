import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useServersStore } from '@/stores/servers'
import api from '@/api/client'

vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn(),
        post: vi.fn(),
        delete: vi.fn(),
    },
}))

describe('Servers Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    describe('initial state', () => {
        it('starts with empty servers array', () => {
            const store = useServersStore()
            expect(store.servers).toEqual([])
            expect(store.loading).toBe(false)
            expect(store.error).toBeNull()
        })
    })

    describe('fetchServers', () => {
        it('fetches servers and updates state', async () => {
            const mockServers = [
                {
                    id: '1',
                    name: 'test-server',
                    url: 'http://localhost:3000',
                    transport: 'http',
                    enabled: true,
                    auth_type: 'none',
                    status: 'healthy',
                    last_health_check: '2024-01-01T00:00:00Z',
                    health_error: null,
                    proxy_url: 'http://localhost:8080/mcp/org/test-server',
                    created_at: '2024-01-01T00:00:00Z',
                    updated_at: '2024-01-01T00:00:00Z',
                },
            ]
            vi.mocked(api.get).mockResolvedValue({ data: mockServers })

            const store = useServersStore()
            await store.fetchServers()

            expect(api.get).toHaveBeenCalledWith('/servers')
            expect(store.servers).toEqual(mockServers)
            expect(store.loading).toBe(false)
            expect(store.error).toBeNull()
        })

        it('handles fetch error', async () => {
            vi.mocked(api.get).mockRejectedValue(new Error('Network error'))

            const store = useServersStore()
            await store.fetchServers()

            expect(store.error).toBeTruthy()
            expect(store.loading).toBe(false)
        })

        it('handles error with response message', async () => {
            vi.mocked(api.get).mockRejectedValue({
                response: { data: { message: 'Custom error message' } },
            })

            const store = useServersStore()
            await store.fetchServers()

            expect(store.error).toBe('Custom error message')
        })
    })

    describe('createServer', () => {
        it('creates server and adds to state', async () => {
            const newServer = {
                id: '2',
                name: 'new-server',
                url: 'http://localhost:4000',
                transport: 'http',
                enabled: true,
                auth_type: 'none',
                status: 'pending_health',
                last_health_check: null,
                health_error: null,
                proxy_url: 'http://localhost:8080/mcp/org/new-server',
                created_at: '2024-01-02T00:00:00Z',
                updated_at: '2024-01-02T00:00:00Z',
            }
            vi.mocked(api.post).mockResolvedValue({ data: newServer })

            const store = useServersStore()
            const result = await store.createServer({
                name: 'new-server',
                url: 'http://localhost:4000',
            })

            expect(api.post).toHaveBeenCalledWith('/servers', {
                name: 'new-server',
                url: 'http://localhost:4000',
            })
            expect(result).toEqual(newServer)
            expect(store.servers[0]).toEqual(newServer)
        })

        it('creates server with transport option', async () => {
            const newServer = {
                id: '3',
                name: 'sse-server',
                url: 'http://localhost:5000',
                transport: 'sse',
            }
            vi.mocked(api.post).mockResolvedValue({ data: newServer })

            const store = useServersStore()
            await store.createServer({
                name: 'sse-server',
                url: 'http://localhost:5000',
                transport: 'sse',
            })

            expect(api.post).toHaveBeenCalledWith('/servers', {
                name: 'sse-server',
                url: 'http://localhost:5000',
                transport: 'sse',
            })
        })
    })

    describe('deleteServer', () => {
        it('removes server from state', async () => {
            const server = {
                id: '1',
                name: 'test-server',
                url: 'http://localhost:3000',
                transport: 'http',
                enabled: true,
                auth_type: 'none',
                status: 'healthy' as const,
                last_health_check: null,
                health_error: null,
                proxy_url: 'http://localhost:8080/mcp/org/test-server',
                created_at: '2024-01-01T00:00:00Z',
                updated_at: '2024-01-01T00:00:00Z',
            }
            vi.mocked(api.get).mockResolvedValue({ data: [server] })
            vi.mocked(api.delete).mockResolvedValue({})

            const store = useServersStore()
            await store.fetchServers()
            expect(store.servers.length).toBe(1)

            await store.deleteServer('test-server')

            expect(api.delete).toHaveBeenCalledWith('/servers/test-server')
            expect(store.servers.length).toBe(0)
        })
    })

    describe('testServer', () => {
        it('returns test result', async () => {
            const testResult = {
                success: true,
                message: 'Connection successful',
                latency_ms: 50,
                status_code: 200,
                tools: [{ name: 'tool1', description: 'A test tool' }],
            }
            vi.mocked(api.post).mockResolvedValue({ data: testResult })

            const store = useServersStore()
            const result = await store.testServer('test-server')

            expect(api.post).toHaveBeenCalledWith('/servers/test-server/test')
            expect(result).toEqual(testResult)
        })
    })

    describe('getServerByName', () => {
        it('returns server by name', async () => {
            const servers = [
                {
                    id: '1',
                    name: 'server-1',
                    url: 'http://localhost:3000',
                    transport: 'http',
                    enabled: true,
                    auth_type: 'none',
                    status: 'healthy' as const,
                    last_health_check: null,
                    health_error: null,
                    proxy_url: 'http://localhost:8080/mcp/org/server-1',
                    created_at: '2024-01-01T00:00:00Z',
                    updated_at: '2024-01-01T00:00:00Z',
                },
                {
                    id: '2',
                    name: 'server-2',
                    url: 'http://localhost:4000',
                    transport: 'sse',
                    enabled: false,
                    auth_type: 'bearer',
                    status: 'disabled' as const,
                    last_health_check: null,
                    health_error: null,
                    proxy_url: 'http://localhost:8080/mcp/org/server-2',
                    created_at: '2024-01-02T00:00:00Z',
                    updated_at: '2024-01-02T00:00:00Z',
                },
            ]
            vi.mocked(api.get).mockResolvedValue({ data: servers })

            const store = useServersStore()
            await store.fetchServers()

            expect(store.getServerByName('server-1')?.id).toBe('1')
            expect(store.getServerByName('server-2')?.id).toBe('2')
            expect(store.getServerByName('nonexistent')).toBeUndefined()
        })
    })
})
