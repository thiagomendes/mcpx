import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'
import { ERROR_MESSAGES } from '@/constants'

export interface Server {
    id: string
    name: string
    url: string
    transport: string
    enabled: boolean
    auth_type: string
    status: 'healthy' | 'unhealthy' | 'pending_auth' | 'pending_health' | 'disabled'
    last_health_check: string | null
    health_error: string | null
    proxy_url: string
    created_at: string
    updated_at: string
}

export interface ToolInfo {
    name: string
    description: string | null
}

export interface TestResult {
    success: boolean
    message: string
    latency_ms: number
    status_code: number | null
    tools: ToolInfo[] | null
}

export const useServersStore = defineStore('servers', () => {
    const servers = ref<Server[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    async function fetchServers() {
        loading.value = true
        error.value = null
        try {
            const response = await api.get('/servers')
            servers.value = response.data
        } catch (e: any) {
            error.value = e.response?.data?.message || ERROR_MESSAGES.FETCH_SERVERS_FAILED
        } finally {
            loading.value = false
        }
    }

    async function createServer(data: { name: string; url: string; transport?: string }) {
        const response = await api.post('/servers', data)
        servers.value.unshift(response.data)
        return response.data
    }

    async function deleteServer(name: string) {
        await api.delete(`/servers/${name}`)
        servers.value = servers.value.filter(s => s.name !== name)
    }

    async function testServer(name: string): Promise<TestResult> {
        const response = await api.post(`/servers/${name}/test`)
        return response.data
    }

    function getServerByName(name: string): Server | undefined {
        return servers.value.find(s => s.name === name)
    }

    return {
        servers,
        loading,
        error,
        fetchServers,
        createServer,
        deleteServer,
        testServer,
        getServerByName,
    }
})
