<template>
  <DashboardLayout>
    <div class="flex items-center justify-between mb-8">
      <div>
        <h1 class="text-2xl font-bold mb-2">Servers</h1>
        <p class="text-gray-400">Manage your MCP servers</p>
      </div>
      <router-link to="/servers/new" class="btn btn-primary flex items-center gap-2">
        <PlusIcon class="w-5 h-5" />
        Add Server
      </router-link>
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
      <router-link to="/servers/new" class="btn btn-primary inline-flex items-center gap-2">
        <PlusIcon class="w-5 h-5" />
        Add Your First Server
      </router-link>
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
            <span :class="server.enabled ? 'badge badge-success' : 'badge badge-warning'">
              {{ server.enabled ? 'Active' : 'Disabled' }}
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
import { onMounted } from 'vue'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useServersStore } from '@/stores/servers'
import { 
  ServerIcon, 
  PlusIcon, 
  ClipboardDocumentIcon, 
  TrashIcon 
} from '@heroicons/vue/24/outline'

const serversStore = useServersStore()

onMounted(() => {
  serversStore.fetchServers()
})

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
