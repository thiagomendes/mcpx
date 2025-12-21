<template>
  <DashboardLayout>
    <div v-if="server" class="max-w-3xl">
      <!-- Header -->
      <div class="flex items-center justify-between mb-8">
        <div>
          <div class="flex items-center gap-3 mb-2">
            <h1 class="text-2xl font-bold">{{ server.name }}</h1>
            <span :class="statusBadgeClass">
              {{ statusLabel }}
            </span>
          </div>
          <p class="text-gray-400 font-mono text-sm">{{ server.url }}</p>
        </div>
        <div class="flex gap-2">
          <button @click="handleDelete" class="btn btn-ghost text-red-400 flex items-center gap-2">
            <TrashIcon class="w-5 h-5" />
            Delete
          </button>
        </div>
      </div>

      <!-- Test Result -->
      <div v-if="testResult" :class="testResult.success ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'" class="px-4 py-3 rounded-lg mb-6 flex items-center gap-2">
        <CheckCircleIcon v-if="testResult.success" class="w-5 h-5" />
        <ExclamationCircleIcon v-else class="w-5 h-5" />
        {{ cleanErrorMessage(testResult.message) }} ({{ testResult.latency_ms }}ms)
      </div>

      <!-- Tools List (collapsible, shown after successful test) -->
      <div v-if="testResult?.success && testResult.tools?.length" class="card mb-6">
        <button 
          @click="toolsExpanded = !toolsExpanded" 
          class="w-full font-semibold flex items-center justify-between cursor-pointer hover:text-primary transition-colors"
        >
          <div class="flex items-center gap-2">
            <CommandLineIcon class="w-5 h-5 text-primary" />
            Available Tools ({{ testResult.tools.length }})
          </div>
          <ChevronDownIcon :class="['w-5 h-5 transition-transform', toolsExpanded ? 'rotate-180' : '']" />
        </button>
        <div v-if="toolsExpanded" class="mt-4 space-y-3">
          <div v-for="tool in testResult.tools" :key="tool.name" class="bg-background-darker px-4 py-3 rounded-lg">
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
        <p class="text-gray-400 text-sm mb-3">Use this URL in Claude Desktop or other MCP clients:</p>
        <div class="flex items-center gap-2">
          <code class="flex-1 bg-background-darker px-4 py-2 rounded font-mono text-sm text-primary">
            {{ server.proxy_url }}
          </code>
          <button @click="copyProxyUrl" class="btn btn-secondary flex items-center gap-2">
            <ClipboardDocumentIcon class="w-5 h-5" />
            Copy
          </button>
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

      <!-- OAuth Authorization (for oauth_auto servers) -->
      <div v-if="server.auth_type === 'oauth_auto'" class="card mb-6">
        <h3 class="font-semibold mb-3 flex items-center gap-2">
          <LockClosedIcon class="w-5 h-5 text-primary" />
          OAuth Authorization
        </h3>
        <div v-if="oauthStatus?.connected && !isTokenExpired" class="bg-green-500/20 text-green-400 px-4 py-3 rounded-lg mb-4 flex items-center gap-2">
          <CheckCircleIcon class="w-5 h-5" />
          Connected - Token expires {{ oauthStatus.expires_at ? new Date(oauthStatus.expires_at).toLocaleString() : 'never' }}
        </div>
        <div v-else-if="oauthStatus?.connected && isTokenExpired" class="bg-red-500/20 text-red-400 px-4 py-3 rounded-lg mb-4 flex items-center gap-2">
          <ExclamationCircleIcon class="w-5 h-5" />
          Token expired - Click "Re-authorize" to reconnect
        </div>
        <div v-else class="bg-yellow-500/20 text-yellow-400 px-4 py-3 rounded-lg mb-4 flex items-center gap-2">
          <ExclamationTriangleIcon class="w-5 h-5" />
          Not authorized - Click "Authorize" to connect with OAuth
        </div>
        <button 
          @click="handleAuthorize" 
          :disabled="authorizing"
          class="btn btn-primary flex items-center gap-2"
        >
          <ShieldCheckIcon class="w-5 h-5" />
          {{ authorizing ? 'Authorizing...' : (oauthStatus?.connected ? 'Re-authorize' : 'Authorize') }}
        </button>
        <p v-if="oauthError" class="text-red-400 text-sm mt-3">{{ oauthError }}</p>
      </div>

      <!-- Health Check Info -->
      <div class="card mb-6">
        <h3 class="font-semibold mb-3 flex items-center gap-2">
          <SignalIcon class="w-5 h-5 text-primary" />
          Health Check
        </h3>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span class="text-gray-400">Status:</span>
            <span :class="statusLabel === 'Healthy' ? 'text-green-400' : statusLabel === 'Unhealthy' ? 'text-red-400' : 'text-yellow-400'">
              {{ statusLabel }}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Last Check:</span>
            <span>{{ server.last_health_check ? new Date(server.last_health_check).toLocaleString() : 'Never' }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Next Check:</span>
            <span class="text-primary font-mono">{{ nextHealthCheckCountdown }}</span>
          </div>
          <div v-if="server.health_error" class="mt-2 p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
            <p class="text-red-400 text-xs">{{ server.health_error }}</p>
          </div>
        </div>
      </div>

      <!-- Tool Governance -->
      <ToolGovernance :server-name="server.name" :available-tools="testResult?.tools || []" />

      <!-- Server Info -->
      <div class="card">
        <h3 class="font-semibold mb-3 flex items-center gap-2">
          <InformationCircleIcon class="w-5 h-5 text-primary" />
          Server Details
        </h3>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span class="text-gray-400">Transport:</span>
            <span>{{ server.transport }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Authentication:</span>
            <span class="capitalize">{{ server.auth_type?.replace('_', ' ') || 'None' }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Created:</span>
            <span>{{ new Date(server.created_at).toLocaleString() }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Updated:</span>
            <span>{{ new Date(server.updated_at).toLocaleString() }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading -->
    <div v-else class="text-center py-12">
      <div class="text-gray-400">Loading server...</div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import ToolGovernance from '@/components/ui/ToolGovernance.vue'
import { useServersStore, type Server, type TestResult } from '@/stores/servers'
import api from '@/api/client'
import { startOAuthFlow } from '@/lib/mcp'
import { 
  SignalIcon, 
  TrashIcon, 
  CheckCircleIcon, 
  ExclamationCircleIcon, 
  ExclamationTriangleIcon,
  LinkIcon, 
  ClipboardDocumentIcon, 
  DocumentTextIcon, 
  InformationCircleIcon,
  LockClosedIcon,
  ShieldCheckIcon,
  CommandLineIcon,
  ChevronDownIcon
} from '@heroicons/vue/24/outline'

const route = useRoute()
const router = useRouter()
const serversStore = useServersStore()

const server = ref<Server | null>(null)
const testResult = ref<TestResult | null>(null)
const testing = ref(false)
const toolsExpanded = ref(false)
const oauthStatus = ref<{ connected: boolean; expires_at: string | null } | null>(null)
const authorizing = ref(false)
const oauthError = ref<string | null>(null)
const countdownTick = ref(0) // For triggering countdown updates
let countdownInterval: ReturnType<typeof setInterval> | null = null

const HEALTH_CHECK_INTERVAL_SECONDS = 300 // 5 minutes

const claudeConfig = computed(() => {
  if (!server.value) return ''
  return JSON.stringify({
    mcpServers: {
      [server.value.name]: {
        url: server.value.proxy_url
      }
    }
  }, null, 2)
})

const statusLabel = computed(() => {
  if (!server.value) return ''
  switch (server.value.status) {
    case 'healthy':
    case 'active': return 'Healthy'
    case 'unhealthy': return 'Unhealthy'
    case 'pending_auth': return 'Pending Auth'
    case 'pending_health': return 'Waiting Check'
    case 'disabled': return 'Disabled'
    default: return server.value.status || 'Unknown'
  }
})

const statusBadgeClass = computed(() => {
  if (!server.value) return 'badge'
  switch (server.value.status) {
    case 'healthy':
    case 'active': return 'badge badge-success'
    case 'unhealthy': return 'badge badge-error'
    case 'pending_auth': return 'badge badge-warning'
    case 'pending_health': return 'badge badge-info'
    case 'disabled': return 'badge badge-error'
    default: return 'badge badge-warning'
  }
})

const nextHealthCheckCountdown = computed(() => {
  // Trigger reactivity on tick
  countdownTick.value
  
  if (!server.value?.last_health_check) return 'Pending...'
  
  const lastCheck = new Date(server.value.last_health_check).getTime()
  const nextCheck = lastCheck + (HEALTH_CHECK_INTERVAL_SECONDS * 1000)
  const now = Date.now()
  const remaining = Math.max(0, nextCheck - now)
  
  if (remaining === 0) return 'Any moment...'
  
  const minutes = Math.floor(remaining / 60000)
  const seconds = Math.floor((remaining % 60000) / 1000)
  
  return `${minutes}m ${seconds.toString().padStart(2, '0')}s`
})

const isTokenExpired = computed(() => {
  if (!oauthStatus.value?.expires_at) return false
  return new Date(oauthStatus.value.expires_at) < new Date()
})

function cleanErrorMessage(message: string): string {
  // Extract user-friendly message from verbose error
  if (message.includes('AuthRequired')) {
    return 'Authentication required - token expired or invalid'
  }
  if (message.includes('Connection failed')) {
    return 'Unable to connect to server'
  }
  if (message.includes('timeout')) {
    return 'Connection timed out'
  }
  // Truncate very long messages
  if (message.length > 80) {
    return message.substring(0, 77) + '...'
  }
  return message
}

onMounted(async () => {
  await serversStore.fetchServers()
  server.value = serversStore.getServerByName(route.params.name as string) || null
  
  // Check OAuth status for oauth_auto servers
  if (server.value?.auth_type === 'oauth_auto') {
    try {
      const response = await api.get(`/servers/${server.value.name}/oauth/status`)
      oauthStatus.value = response.data
      
  
      if (oauthStatus.value?.connected) {
        await handleTest()
      }
    } catch (e) {
      oauthStatus.value = { connected: false, expires_at: null }
    }
  } else {

    await handleTest()
  }
  
  // Check for OAuth callback success
  if (route.query.oauth === 'success') {
    oauthStatus.value = { connected: true, expires_at: null }

    if (server.value) {
      const response = await api.get(`/servers/${server.value.name}/oauth/status`)
      oauthStatus.value = response.data
  
      await handleTest()
    }
  }
  
  // Start countdown timer
  countdownInterval = setInterval(() => {
    countdownTick.value++
  }, 1000)
})

onUnmounted(() => {
  if (countdownInterval) {
    clearInterval(countdownInterval)
  }
})

async function handleAuthorize() {
  if (!server.value) return
  authorizing.value = true
  oauthError.value = null
  
  try {

    const result = await startOAuthFlow(server.value.name, server.value.url)
    
    if (result.success) {
  
      const maxAttempts = 60 // 60 seconds max wait
      let attempts = 0
      let oauthComplete = false
      
      while (attempts < maxAttempts && !oauthComplete) {
        await new Promise(resolve => setTimeout(resolve, 1000))
        attempts++
        
        try {
          const statusResponse = await api.get(`/servers/${server.value!.name}/oauth/status`)
          if (statusResponse.data.connected) {
            oauthStatus.value = statusResponse.data
            oauthComplete = true
          }
        } catch {
      
        }
      }
      
      if (oauthComplete) {
    
        await serversStore.fetchServers()
        server.value = serversStore.getServerByName(server.value!.name) || null
        
    
        await handleTest()
      } else {
        oauthError.value = 'OAuth flow timeout - popup may have been closed'
      }
    } else {
      oauthError.value = result.error || 'OAuth authorization failed'
    }
  } catch (e: any) {
    oauthError.value = e.message || 'Failed to start OAuth flow'
  } finally {
    authorizing.value = false
  }
}

async function handleTest() {
  if (!server.value) return
  testing.value = true
  testResult.value = null
  
  try {
    testResult.value = await serversStore.testServer(server.value.name)
  } catch (e: any) {
    testResult.value = {
      success: false,
      message: e.response?.data || 'Test failed',
      latency_ms: 0,
      status_code: null,
    }
  } finally {
    testing.value = false
  }
}

async function handleDelete() {
  if (!server.value) return
  if (confirm(`Delete server "${server.value.name}"? This cannot be undone.`)) {
    await serversStore.deleteServer(server.value.name)
    router.push('/servers')
  }
}

function copyProxyUrl() {
  if (!server.value) return
  navigator.clipboard.writeText(server.value.proxy_url)
  alert('Proxy URL copied!')
}

function copyClaudeConfig() {
  navigator.clipboard.writeText(claudeConfig.value)
  alert('Config copied!')
}
</script>
