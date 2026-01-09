// Global test setup - mocks that apply to all tests
import { vi } from 'vitest'

// Mock API client globally to prevent actual network calls
vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn().mockImplementation((url: string) => {
            // Return appropriate mock data based on URL
            if (url === '/limits') {
                return Promise.resolve({
                    data: {
                        gateways: { current: 0, max: 10, can_create: true },
                        servers: { current: 0, max: 10, can_create: true },
                        service_accounts: { current: 0, max: 10, can_create: true },
                    }
                })
            }
            if (url === '/service-accounts') {
                return Promise.resolve({ data: [] })
            }
            if (url === '/identity-providers') {
                return Promise.resolve({ data: [] })
            }
            return Promise.resolve({ data: {} })
        }),
        post: vi.fn().mockResolvedValue({ data: {} }),
        put: vi.fn().mockResolvedValue({ data: {} }),
        patch: vi.fn().mockResolvedValue({ data: {} }),
        delete: vi.fn().mockResolvedValue({ data: {} }),
    }
}))
