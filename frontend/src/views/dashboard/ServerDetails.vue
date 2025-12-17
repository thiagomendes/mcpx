<template>
  <DashboardLayout>
    <div v-if="server" class="max-w-3xl">
      <!-- Header -->
      <div class="flex items-center justify-between mb-8">
        <div>
          <div class="flex items-center gap-3 mb-2">
            <h1 class="text-2xl font-bold">{{ server.name }}</h1>
            <span :class="server.enabled ? 'badge badge-success' : 'badge badge-warning'">
              {{ server.enabled ? 'Active' : 'Disabled' }}
            </span>
          </div>
          <p class="text-gray-400 font-mono text-sm">{{ server.url }}</p>
        </div>
        <div class="flex gap-2">
          <button @click="handleTest" :disabled="testing" class="btn btn-secondary flex items-center gap-2">
            <SignalIcon class="w-5 h-5" />
            {{ testing ? 'Testing...' : 'Test Connection' }}
          </button>
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
        {{ testResult.message }} ({{ testResult.latency_ms }}ms)
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
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useServersStore, type Server, type TestResult } from '@/stores/servers'
import { 
  SignalIcon, 
  TrashIcon, 
  CheckCircleIcon, 
  ExclamationCircleIcon, 
  LinkIcon, 
  ClipboardDocumentIcon, 
  DocumentTextIcon, 
  InformationCircleIcon 
} from '@heroicons/vue/24/outline'

const route = useRoute()
const router = useRouter()
const serversStore = useServersStore()

const server = ref<Server | null>(null)
const testResult = ref<TestResult | null>(null)
const testing = ref(false)

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

onMounted(async () => {
  await serversStore.fetchServers()
  server.value = serversStore.getServerByName(route.params.name as string) || null
})

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
