import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface GatewayServer {
    name: string
    priority: number
}

export interface Gateway {
    id: string
    name: string
    slug: string
    enabled: boolean
    proxy_url: string
    servers: GatewayServer[]
    created_at: string
    updated_at: string
}

export const useGatewaysStore = defineStore('gateways', () => {
    const gateways = ref<Gateway[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    async function fetchGateways() {
        loading.value = true
        error.value = null
        try {
            const response = await api.get('/gateways')
            gateways.value = response.data
        } catch {
            error.value = 'Failed to fetch gateways'
        } finally {
            loading.value = false
        }
    }

    async function createGateway(data: { name: string; slug: string; servers?: string[] }) {
        const response = await api.post('/gateways', data)
        gateways.value.unshift(response.data)
        return response.data
    }

    async function updateGateway(slug: string, data: { name?: string; slug?: string; enabled?: boolean }) {
        const response = await api.put(`/gateways/${slug}`, data)
        const index = gateways.value.findIndex(g => g.slug === slug)
        if (index !== -1) {
            gateways.value[index] = response.data
        }
        return response.data
    }

    async function deleteGateway(slug: string) {
        await api.delete(`/gateways/${slug}`)
        gateways.value = gateways.value.filter(g => g.slug !== slug)
    }

    async function addServerToGateway(slug: string, serverName: string) {
        await api.post(`/gateways/${slug}/servers`, { server_name: serverName })
        await fetchGateways()
    }

    async function removeServerFromGateway(slug: string, serverName: string) {
        await api.delete(`/gateways/${slug}/servers/${serverName}`)
        await fetchGateways()
    }

    function getGatewayBySlug(slug: string): Gateway | undefined {
        return gateways.value.find(g => g.slug === slug)
    }

    return {
        gateways,
        loading,
        error,
        fetchGateways,
        createGateway,
        updateGateway,
        deleteGateway,
        addServerToGateway,
        removeServerFromGateway,
        getGatewayBySlug,
    }
})
