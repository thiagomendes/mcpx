<template>
  <DashboardLayout>
    <div class="max-w-2xl">
      <div class="mb-8">
        <h1 class="text-2xl font-bold mb-2">Add Server</h1>
        <p class="text-gray-400">Connect a new MCP server</p>
      </div>

      <form @submit.prevent="handleSubmit" class="card space-y-6">
        <!-- Name -->
        <div>
          <label class="block text-sm font-medium mb-2">Server Name</label>
          <input 
            v-model="form.name"
            type="text" 
            class="input"
            placeholder="my-server"
            required
          />
          <p class="text-gray-500 text-sm mt-1">Use letters, numbers, hyphens, and underscores only</p>
        </div>

        <!-- URL -->
        <div>
          <label class="block text-sm font-medium mb-2">Server URL</label>
          <input 
            v-model="form.url"
            type="url" 
            class="input"
            placeholder="https://api.example.com/mcp"
            required
          />
          <p class="text-gray-500 text-sm mt-1">The MCP endpoint URL of your server</p>
        </div>

        <!-- Transport -->
        <div>
          <label class="block text-sm font-medium mb-2">Transport</label>
          <select v-model="form.transport" class="input">
            <option value="streamable-http">Streamable HTTP (recommended)</option>
            <option value="sse">SSE (legacy)</option>
          </select>
        </div>

        <!-- Error -->
        <div v-if="error" class="bg-red-500/20 text-red-400 px-4 py-2 rounded-lg flex items-center gap-2">
          <ExclamationCircleIcon class="w-5 h-5" />
          {{ error }}
        </div>

        <!-- Test Result -->
        <div v-if="testResult" :class="testResult.success ? 'bg-green-500/20 text-green-400' : 'bg-yellow-500/20 text-yellow-400'" class="px-4 py-2 rounded-lg flex items-center gap-2">
          <CheckCircleIcon v-if="testResult.success" class="w-5 h-5" />
          <ExclamationTriangleIcon v-else class="w-5 h-5" />
          {{ testResult.message }} ({{ testResult.latency_ms }}ms)
        </div>

        <!-- Actions -->
        <div class="flex gap-4 pt-4">
          <button 
            type="button" 
            @click="handleTest"
            :disabled="testing || !form.url"
            class="btn btn-secondary flex items-center gap-2"
          >
            <SignalIcon class="w-5 h-5" />
            {{ testing ? 'Testing...' : 'Test Connection' }}
          </button>
          <button 
            type="submit" 
            :disabled="submitting"
            class="btn btn-primary flex items-center gap-2"
          >
            <PlusIcon class="w-5 h-5" />
            {{ submitting ? 'Creating...' : 'Create Server' }}
          </button>
        </div>
      </form>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useServersStore, type TestResult } from '@/stores/servers'
import { 
  ExclamationCircleIcon, 
  CheckCircleIcon, 
  ExclamationTriangleIcon, 
  SignalIcon, 
  PlusIcon 
} from '@heroicons/vue/24/outline'

const router = useRouter()
const serversStore = useServersStore()

const form = reactive({
  name: '',
  url: '',
  transport: 'streamable-http',
})

const error = ref<string | null>(null)
const testResult = ref<TestResult | null>(null)
const testing = ref(false)
const submitting = ref(false)

async function handleTest() {
  if (!form.url) return
  
  testing.value = true
  testResult.value = null
  
  try {
    // Create a temporary test by calling the URL directly
    const response = await fetch(form.url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json, text/event-stream',
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'initialize',
        params: {
          protocolVersion: '2025-11-05',
          capabilities: {},
          clientInfo: { name: 'mcpx-test', version: '1.0.0' }
        }
      })
    })
    
    testResult.value = {
      success: response.ok,
      message: response.ok ? 'Connection successful!' : `Server returned ${response.status}`,
      latency_ms: 0,
      status_code: response.status,
    }
  } catch (e: any) {
    testResult.value = {
      success: false,
      message: `Connection failed: ${e.message}`,
      latency_ms: 0,
      status_code: null,
    }
  } finally {
    testing.value = false
  }
}

async function handleSubmit() {
  error.value = null
  submitting.value = true
  
  try {
    await serversStore.createServer(form)
    router.push('/servers')
  } catch (e: any) {
    error.value = e.response?.data || 'Failed to create server'
  } finally {
    submitting.value = false
  }
}
</script>
