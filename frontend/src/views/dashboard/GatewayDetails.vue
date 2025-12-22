<template>
  <DashboardLayout>
    <div v-if="gateway" class="max-w-3xl">
      <!-- Header -->
      <div class="flex items-center justify-between mb-8">
        <div>
          <div class="flex items-center gap-3 mb-2">
            <h1 class="text-2xl font-bold">{{ gateway.name }}</h1>
            <span :class="gateway.enabled ? 'badge badge-success' : 'badge badge-error'">
              {{ gateway.enabled ? 'Enabled' : 'Disabled' }}
            </span>
          </div>
          <p class="text-gray-400 font-mono text-sm">Slug: {{ gateway.slug }}</p>
        </div>
        <div class="flex gap-2">
          <button 
            @click="toggleEnabled" 
            :class="gateway.enabled ? 'btn btn-ghost text-yellow-400' : 'btn btn-secondary'"
          >
            {{ gateway.enabled ? 'Disable' : 'Enable' }}
          </button>
          <button @click="handleDelete" class="btn btn-ghost text-red-400 flex items-center gap-2">
            <TrashIcon class="w-5 h-5" />
            Delete
          </button>
        </div>
      </div>

      <!-- Tools List -->
      <div class="card mb-6">
        <button 
          @click="toolsExpanded = !toolsExpanded" 
          class="w-full font-semibold flex items-center justify-between cursor-pointer hover:text-primary transition-colors"
        >
          <div class="flex items-center gap-2">
            <CommandLineIcon class="w-5 h-5 text-primary" />
            <span v-if="loadingTools" class="flex items-center gap-2">
              <svg class="animate-spin h-4 w-4 text-primary" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Loading tools from servers...
            </span>
            <span v-else-if="toolsError">Failed to load tools</span>
            <span v-else>Available Tools ({{ tools.length }})</span>
          </div>
          <ChevronDownIcon :class="['w-5 h-5 transition-transform', toolsExpanded ? 'rotate-180' : '']" />
        </button>
        <div v-if="toolsExpanded" class="mt-4 space-y-3">
          <div v-if="loadingTools" class="text-gray-400 text-sm text-center py-4">
            <svg class="animate-spin h-8 w-8 text-primary mx-auto mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Connecting to servers and aggregating tools...
          </div>
          <div v-else-if="toolsError" class="text-red-400 text-sm">{{ toolsError }}</div>
          <div v-else-if="tools.length === 0" class="text-gray-400 text-sm">No tools available. Add servers to the gateway first.</div>
          <div v-else v-for="tool in tools" :key="tool.name" class="bg-background-darker px-4 py-3 rounded-lg">
            <div class="font-mono text-sm text-primary">{{ tool.name }}</div>
            <div v-if="tool.description" class="text-gray-400 text-sm mt-1">{{ tool.description }}</div>
          </div>
        </div>
      </div>

      <!-- Proxy URL -->
      <div class="card mb-6">
        <h3 class="font-semibold mb-3 flex items-center gap-2">
          <LinkIcon class="w-5 h-5 text-primary" />
          Proxy URL
        </h3>
        <p class="text-gray-400 text-sm mb-3">Use this URL to access all tools from the gateway's servers:</p>
        <div class="flex items-center gap-2">
          <code class="flex-1 bg-background-darker px-4 py-2 rounded font-mono text-sm text-primary">
            {{ gateway.proxy_url }}
          </code>
          <button @click="copyProxyUrl" class="btn btn-secondary flex items-center gap-2">
            <ClipboardDocumentIcon class="w-5 h-5" />
            Copy
          </button>
        </div>
      </div>

      <!-- Servers in Gateway -->
      <div class="card mb-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="font-semibold flex items-center gap-2">
            <ServerIcon class="w-5 h-5 text-primary" />
            Servers ({{ gateway.servers.length }})
          </h3>
          <button @click="showAddServerModal = true" class="btn btn-secondary text-sm flex items-center gap-2">
            <PlusIcon class="w-4 h-4" />
            Add Server
          </button>
        </div>
        
        <div v-if="gateway.servers.length === 0" class="text-gray-400 text-center py-8">
          No servers added yet. Add servers to aggregate their tools.
        </div>
        
        <div v-else class="space-y-3">
          <div 
            v-for="server in gateway.servers" 
            :key="server.name"
            class="flex items-center justify-between bg-background-darker px-4 py-3 rounded-lg"
          >
            <div>
              <div class="font-mono text-sm font-semibold">{{ server.name }}</div>
              <div class="text-gray-400 text-xs">Priority: {{ server.priority }}</div>
            </div>
            <button 
              @click="removeServer(server.name)" 
              class="text-red-400 hover:text-red-300"
              title="Remove server"
            >
              <XMarkIcon class="w-5 h-5" />
            </button>
          </div>
        </div>
      </div>

      <!-- Claude Desktop Config -->
      <div class="card mb-6">
        <h3 class="font-semibold mb-3 flex items-center gap-2">
          <DocumentTextIcon class="w-5 h-5 text-primary" />
          Claude Desktop Configuration
        </h3>
        <p class="text-gray-400 text-sm mb-3">Add this to your Claude Desktop config:</p>
        <pre class="bg-background-darker px-4 py-3 rounded font-mono text-sm overflow-x-auto">{{ claudeConfig }}</pre>
        <button @click="copyClaudeConfig" class="btn btn-secondary mt-3 flex items-center gap-2">
          <ClipboardDocumentIcon class="w-5 h-5" />
          Copy Config
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-else class="text-center py-12">
      <div class="text-gray-400">Loading gateway...</div>
    </div>

    <!-- Add Server Modal -->
    <div v-if="showAddServerModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div class="card w-full max-w-md">
        <h3 class="text-lg font-semibold mb-4">Add Server to Gateway</h3>
        <div class="mb-4">
          <label class="block text-sm font-medium mb-2">Select Server</label>
          <select v-model="selectedServer" class="input w-full">
            <option value="">Choose a server...</option>
            <option 
              v-for="server in availableServers" 
              :key="server.name" 
              :value="server.name"
            >
              {{ server.name }}
            </option>
          </select>
        </div>
        <div class="flex gap-3 justify-end">
          <button @click="showAddServerModal = false" class="btn btn-ghost">Cancel</button>
          <button 
            @click="addServer" 
            class="btn btn-primary" 
            :disabled="!selectedServer || addingServer"
          >
            {{ addingServer ? 'Adding...' : 'Add Server' }}
          </button>
        </div>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useGatewaysStore } from '@/stores/gateways'
import { useServersStore } from '@/stores/servers'
import { 
  TrashIcon, 
  LinkIcon, 
  ClipboardDocumentIcon, 
  DocumentTextIcon,
  ServerIcon,
  PlusIcon,
  XMarkIcon,
  CommandLineIcon,
  ChevronDownIcon,
} from '@heroicons/vue/24/outline'
import { useAuthStore } from '@/stores/auth'

interface GatewayTool {
  name: string
  description?: string
}

const route = useRoute()
const router = useRouter()
const gatewaysStore = useGatewaysStore()
const serversStore = useServersStore()
const authStore = useAuthStore()

const showAddServerModal = ref(false)
const selectedServer = ref('')
const addingServer = ref(false)
const tools = ref<GatewayTool[]>([])
const loadingTools = ref(false)
const toolsError = ref<string | null>(null)
const toolsExpanded = ref(true)

const slug = computed(() => route.params.slug as string)
const gateway = computed(() => gatewaysStore.getGatewayBySlug(slug.value))

const availableServers = computed(() => {
  const assignedNames = new Set(gateway.value?.servers.map(s => s.name) || [])
  return serversStore.servers.filter(s => !assignedNames.has(s.name))
})

const claudeConfig = computed(() => {
  if (!gateway.value) return ''
  return JSON.stringify({
    "mcpServers": {
      [gateway.value.slug]: {
        "url": gateway.value.proxy_url,
        "transport": "streamable-http"
      }
    }
  }, null, 2)
})

onMounted(async () => {
  await gatewaysStore.fetchGateways()
  await serversStore.fetchServers()
  // Wait for gateway to be populated
  await nextTick()
  if (gateway.value && gateway.value.servers.length > 0) {
    await loadTools()
  }
})

// Watch for servers being added to reload tools
watch(() => gateway.value?.servers.length, async (newLen, oldLen) => {
  if (newLen && newLen > 0 && newLen !== oldLen) {
    await loadTools()
  }
})

async function loadTools() {
  if (!gateway.value || !authStore.user?.id) return
  loadingTools.value = true
  toolsError.value = null
  
  try {
    const response = await fetch(gateway.value.proxy_url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'tools/list',
        params: {},
        id: 1
      })
    })
    
    const text = await response.text()
    const lines = text.split('\n')
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = JSON.parse(line.slice(6))
        if (data.result?.tools) {
          tools.value = data.result.tools
        }
      }
    }
  } catch {
    toolsError.value = 'Failed to fetch tools from gateway'
  } finally {
    loadingTools.value = false
  }
}

function copyProxyUrl() {
  if (gateway.value) {
    navigator.clipboard.writeText(gateway.value.proxy_url)
    alert('Proxy URL copied to clipboard!')
  }
}

function copyClaudeConfig() {
  navigator.clipboard.writeText(claudeConfig.value)
  alert('Claude config copied to clipboard!')
}

async function toggleEnabled() {
  if (gateway.value) {
    await gatewaysStore.updateGateway(gateway.value.slug, { enabled: !gateway.value.enabled })
    await gatewaysStore.fetchGateways()
  }
}

async function handleDelete() {
  if (gateway.value && confirm(`Delete gateway "${gateway.value.name}"? This cannot be undone.`)) {
    await gatewaysStore.deleteGateway(gateway.value.slug)
    router.push('/gateways')
  }
}

async function addServer() {
  if (!gateway.value || !selectedServer.value) return
  addingServer.value = true
  try {
    await gatewaysStore.addServerToGateway(gateway.value.slug, selectedServer.value)
    showAddServerModal.value = false
    selectedServer.value = ''
  } catch {
    alert('Failed to add server')
  } finally {
    addingServer.value = false
  }
}

async function removeServer(serverName: string) {
  if (!gateway.value) return
  if (confirm(`Remove "${serverName}" from this gateway?`)) {
    await gatewaysStore.removeServerFromGateway(gateway.value.slug, serverName)
  }
}
</script>
