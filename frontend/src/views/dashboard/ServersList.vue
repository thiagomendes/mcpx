<template>
  <DashboardLayout>
    <div class="flex items-center justify-between mb-8">
      <div>
        <h1 class="text-2xl font-bold mb-2">Servers</h1>
        <p class="text-gray-400">Manage your MCP servers</p>
      </div>
      <button 
        v-if="canWrite"
        @click="goToAddServer"
        :disabled="limitReached"
        :class="['btn flex items-center gap-2', limitReached ? 'btn-secondary opacity-50 cursor-not-allowed' : 'btn-primary']"
      >
        <PlusIcon class="w-5 h-5" />
        Add Server
      </button>
    </div>

    <!-- Limit Warning (only for admins) -->
    <div v-if="limitReached && canWrite" class="card bg-amber-500/20 border-amber-500/50 mb-4">
      <div class="flex items-start gap-3">
        <NoSymbolIcon class="w-5 h-5 text-amber-400 shrink-0" />
        <p class="text-gray-300 text-sm">
          You have {{ limitInfo.current }}/{{ limitInfo.max }} servers. Delete one or increase the limit in 
          <router-link to="/settings/configuration" class="text-violet-400 hover:underline">Configuration</router-link>.
        </p>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="serversStore.loading" class="text-center py-12">
      <div class="text-gray-400">Loading servers...</div>
    </div>

    <!-- Empty State -->
    <div v-else-if="serversStore.servers.length === 0" class="card text-center py-12">
      <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-primary/20 flex items-center justify-center">
        <ServerIcon class="w-8 h-8 text-primary" />
      </div>
      <h3 class="text-lg font-semibold mb-2">No servers yet</h3>
      <p class="text-gray-400 mb-6">Connect your first MCP server to get started</p>
      <button 
        @click="goToAddServer"
        :disabled="limitReached"
        :class="['btn inline-flex items-center gap-2', limitReached ? 'btn-secondary opacity-50 cursor-not-allowed' : 'btn-primary']"
      >
        <PlusIcon class="w-5 h-5" />
        Add Your First Server
      </button>
    </div>

    <!-- Server List -->
    <div v-else class="space-y-4">
      <div 
        v-for="server in serversStore.servers" 
        :key="server.id"
        class="card flex items-center justify-between hover:border-primary cursor-pointer"
        @click="$router.push(`/servers/${server.name}`)"
      >
        <div class="flex-1">
          <div class="flex items-center gap-3 mb-1">
            <span class="font-semibold">{{ server.name }}</span>
            <span :class="statusBadgeClass(server.status)">
              {{ statusLabel(server.status) }}
            </span>
          </div>
          <div class="text-gray-400 text-sm font-mono">{{ server.url }}</div>
        </div>
        <div class="flex items-center gap-2">
          <button 
            @click.stop="copyProxyUrl(server.proxy_url)" 
            class="btn btn-ghost text-sm flex items-center gap-1"
            title="Copy proxy URL"
          >
            <ClipboardDocumentIcon class="w-4 h-4" />
            Copy URL
          </button>
          <button 
            v-if="canWrite"
            @click.stop="handleDelete(server.name)" 
            class="btn btn-ghost text-sm text-red-400 hover:text-red-300 flex items-center gap-1"
          >
            <TrashIcon class="w-4 h-4" />
            Delete
          </button>
        </div>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useServersStore } from '@/stores/servers'

import { usePermissions } from '@/composables/usePermissions'
import { 
  ServerIcon, 
  PlusIcon, 
  ClipboardDocumentIcon, 
  TrashIcon,
  NoSymbolIcon
} from '@heroicons/vue/24/outline'

const router = useRouter()
const serversStore = useServersStore()
const { canWrite } = usePermissions()
const limitReached = ref(false)
const limitInfo = ref({ current: 0, max: 0 })

async function checkLimits() {
  try {
    const api = (await import('@/api/client')).default
    const response = await api.get('/limits')
    const data = response.data
    limitInfo.value = { current: data.servers.current, max: data.servers.max }
    limitReached.value = !data.servers.can_create
  } catch (e) {
    console.error('Failed to check limits:', e)
  }
}

function goToAddServer() {
  if (!limitReached.value) {
    router.push('/servers/new')
  }
}

onMounted(() => {
  serversStore.fetchServers()
  checkLimits()
})

function statusLabel(status: string): string {
  switch (status) {
    case 'healthy':
    case 'active': return 'Healthy'
    case 'unhealthy': return 'Unhealthy'
    case 'pending_auth': return 'Pending Auth'
    case 'pending_health': return 'Waiting Check'
    case 'disabled': return 'Disabled'
    default: return status || 'Unknown'
  }
}

function statusBadgeClass(status: string): string {
  switch (status) {
    case 'healthy':
    case 'active': return 'badge badge-success'
    case 'unhealthy': return 'badge badge-error'
    case 'pending_auth': return 'badge badge-warning'
    case 'pending_health': return 'badge badge-info'
    case 'disabled': return 'badge badge-error'
    default: return 'badge badge-warning'
  }
}

function copyProxyUrl(url: string) {
  navigator.clipboard.writeText(url)
  alert('Proxy URL copied to clipboard!')
}

async function handleDelete(name: string) {
  if (confirm(`Delete server "${name}"? This cannot be undone.`)) {
    await serversStore.deleteServer(name)
  }
}
</script>
