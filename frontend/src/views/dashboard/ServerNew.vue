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
          <div class="flex items-center gap-2 mb-2">
            <label class="block text-sm font-medium">Transport</label>
            <InfoTooltip 
              title="Transport Protocol"
              content="How mcpx communicates with the server. Auto-detect tries Streamable HTTP first and falls back to SSE if needed. Most modern MCP servers use Streamable HTTP."
              type="info"
            />
          </div>
          <select v-model="form.transport" class="input">
            <option value="auto">Auto-detect (recommended)</option>
            <option value="streamable-http">Streamable HTTP</option>
            <option value="sse">SSE (legacy)</option>
          </select>
        </div>

        <!-- Authentication -->
        <div class="border-t border-gray-700 pt-6">
          <div class="flex items-center gap-2 mb-4">
            <LockClosedIcon class="w-5 h-5" />
            <h3 class="text-lg font-medium">Authentication</h3>
            <InfoTooltip 
              title="Server Authentication"
              content="Configure how mcpx authenticates with the upstream MCP server. This is NOT the user authentication - it's how mcpx connects to servers on your behalf."
              type="info"
            />
          </div>
          <p class="text-gray-400 text-sm mb-4">How should mcpx authenticate with this server?</p>
          
          <div class="space-y-3">
            <!-- None -->
            <label class="flex items-start gap-3 p-3 rounded-lg border border-gray-700 cursor-pointer hover:bg-gray-800" :class="form.auth_type === 'none' && 'border-indigo-500 bg-gray-800'">
              <input type="radio" v-model="form.auth_type" value="none" class="text-indigo-500 mt-1" />
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-medium">None</span>
                  <InfoTooltip 
                    title="Public Server"
                    content="Use this for servers that don't require any authentication. The server is publicly accessible without credentials."
                    type="info"
                  />
                </div>
                <div class="text-sm text-gray-400">Public server, no authentication required</div>
              </div>
            </label>
            
            <!-- API Key -->
            <label class="flex items-start gap-3 p-3 rounded-lg border border-gray-700 cursor-pointer hover:bg-gray-800" :class="form.auth_type === 'api_key' && 'border-indigo-500 bg-gray-800'">
              <input type="radio" v-model="form.auth_type" value="api_key" class="text-indigo-500 mt-1" />
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-medium">API Key</span>
                  <InfoTooltip 
                    title="API Key Authentication"
                    content="The server expects an API key sent in the X-API-Key header. Common for simple API integrations. You'll need to obtain the key from the server provider."
                    type="tip"
                  />
                </div>
                <div class="text-sm text-gray-400">Send a static API key in the X-API-Key header</div>
              </div>
            </label>
            
            <!-- Bearer Token -->
            <label class="flex items-start gap-3 p-3 rounded-lg border border-gray-700 cursor-pointer hover:bg-gray-800" :class="form.auth_type === 'bearer' && 'border-indigo-500 bg-gray-800'">
              <input type="radio" v-model="form.auth_type" value="bearer" class="text-indigo-500 mt-1" />
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-medium">Bearer Token</span>
                  <InfoTooltip 
                    title="Bearer Token Authentication"
                    content="The server expects a bearer token in the Authorization header. Use this when you have a pre-issued token (like a PAT - Personal Access Token) from the server provider."
                    type="tip"
                  />
                </div>
                <div class="text-sm text-gray-400">Send a static token in the Authorization header</div>
              </div>
            </label>
            
            <!-- OAuth 2.1 -->
            <div class="rounded-lg border border-gray-700" :class="form.auth_type.startsWith('oauth') && 'border-indigo-500 bg-gray-800'">
              <label class="flex items-start gap-3 p-3 cursor-pointer hover:bg-gray-800 rounded-t-lg" @click="form.auth_type = 'oauth_auto'">
                <input type="radio" :checked="form.auth_type.startsWith('oauth')" class="text-indigo-500 mt-1" />
                <div class="flex-1">
                  <div class="flex items-center gap-2">
                    <span class="font-medium">OAuth 2.1</span>
                    <InfoTooltip 
                      title="OAuth 2.1 Authentication"
                      content="Secure authentication using OAuth 2.1 protocol. Supports automatic discovery per MCP spec, client credentials for M2M, or manual configuration for legacy servers."
                      type="info"
                    />
                  </div>
                  <div class="text-sm text-gray-400">Authenticate via OAuth flow</div>
                </div>
              </label>
              
              <!-- OAuth Sub-options -->
              <div v-if="form.auth_type.startsWith('oauth')" class="border-t border-gray-700 p-3 space-y-3 bg-gray-900/50 rounded-b-lg">
                <!-- Auto-Discovery -->
                <label class="flex items-start gap-3 p-3 rounded-lg border border-gray-600 cursor-pointer hover:bg-gray-800" :class="form.auth_type === 'oauth_auto' && 'border-fuchsia-500 bg-gray-800'">
                  <input type="radio" v-model="form.auth_type" value="oauth_auto" class="text-fuchsia-500 mt-1" />
                  <div class="flex-1">
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-fuchsia-400">🔄 Auto-Discovery (MCP Standard)</span>
                      <InfoTooltip 
                        title="Automatic OAuth Discovery"
                        content="Recommended for MCP-compliant servers. mcpx automatically discovers OAuth endpoints from the server's well-known URLs. Uses PKCE for security. You'll be prompted to authorize via popup when connecting."
                        type="success"
                      />
                    </div>
                    <div class="text-sm text-gray-400">Automatically discover OAuth endpoints from server</div>
                  </div>
                </label>
                
                <!-- Client Credentials -->
                <label class="flex items-start gap-3 p-3 rounded-lg border border-gray-600 cursor-pointer hover:bg-gray-800" :class="form.auth_type === 'oauth_client_credentials' && 'border-fuchsia-500 bg-gray-800'">
                  <input type="radio" v-model="form.auth_type" value="oauth_client_credentials" class="text-fuchsia-500 mt-1" />
                  <div class="flex-1">
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-fuchsia-400">🔐 Client Credentials</span>
                      <InfoTooltip 
                        title="Client Credentials Grant"
                        content="For machine-to-machine (M2M) authentication without user interaction. Requires client_id and client_secret from the OAuth provider. The token is obtained directly without a user popup."
                        type="tip"
                      />
                    </div>
                    <div class="text-sm text-gray-400">Server-to-server auth (no user interaction)</div>
                  </div>
                </label>
                
                <!-- Manual Configuration -->
                <label class="flex items-start gap-3 p-3 rounded-lg border border-gray-600 cursor-pointer hover:bg-gray-800" :class="form.auth_type === 'oauth_manual' && 'border-fuchsia-500 bg-gray-800'">
                  <input type="radio" v-model="form.auth_type" value="oauth_manual" class="text-fuchsia-500 mt-1" />
                  <div class="flex-1">
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-fuchsia-400">⚙️ Manual Configuration</span>
                      <InfoTooltip 
                        title="Manual OAuth Setup"
                        content="For legacy servers that don't support OAuth discovery. You'll need to manually enter the authorization URL, token URL, and client credentials. Use this as a fallback when auto-discovery fails."
                        type="warning"
                      />
                    </div>
                    <div class="text-sm text-gray-400">Manually configure OAuth endpoints (legacy)</div>
                  </div>
                </label>
              </div>
            </div>
          </div>
          
          <!-- Conditional fields based on auth type -->
          <div v-if="form.auth_type === 'api_key'" class="mt-4 space-y-4 p-4 bg-gray-900/50 rounded-lg">
            <div>
              <label class="block text-sm font-medium mb-2">API Key</label>
              <input 
                v-model="form.api_key"
                type="password" 
                class="input"
                placeholder="Enter your API key"
              />
            </div>
          </div>
          
          <div v-if="form.auth_type === 'bearer'" class="mt-4 space-y-4 p-4 bg-gray-900/50 rounded-lg">
            <div>
              <label class="block text-sm font-medium mb-2">Bearer Token</label>
              <input 
                v-model="form.bearer_token"
                type="password" 
                class="input"
                placeholder="Enter your bearer token"
              />
            </div>
          </div>
          
          <!-- OAuth Auto-Discovery info -->
          <div v-if="form.auth_type === 'oauth_auto'" class="mt-4 p-4 bg-fuchsia-500/10 border border-fuchsia-500/30 rounded-lg">
            <div class="flex items-start gap-3">
              <SparklesIcon class="w-5 h-5 text-fuchsia-400 shrink-0 mt-0.5" />
              <div>
                <p class="text-sm text-fuchsia-300 font-medium">Automatic OAuth Configuration</p>
                <p class="text-sm text-gray-400 mt-1">After creating the server, click "Authorize" to start the OAuth flow. mcpx will automatically discover the OAuth endpoints and open a popup for you to log in.</p>
              </div>
            </div>
          </div>
          
          <!-- Client Credentials fields -->
          <div v-if="form.auth_type === 'oauth_client_credentials'" class="mt-4 space-y-4 p-4 bg-gray-900/50 rounded-lg">
            <div>
              <label class="block text-sm font-medium mb-2">Client ID</label>
              <input v-model="form.oauth_client_id" type="text" class="input" placeholder="your-client-id" />
            </div>
            <div>
              <label class="block text-sm font-medium mb-2">Client Secret</label>
              <input v-model="form.oauth_client_secret" type="password" class="input" placeholder="your-client-secret" />
            </div>
            <div>
              <label class="block text-sm font-medium mb-2">Token URL (optional - auto-discovered if empty)</label>
              <input v-model="form.oauth_token_url" type="url" class="input" placeholder="https://provider.com/oauth/token" />
            </div>
            <div>
              <label class="block text-sm font-medium mb-2">Scopes</label>
              <input v-model="form.oauth_scopes" type="text" class="input" placeholder="read write" />
            </div>
          </div>
          
          <!-- Manual OAuth fields -->
          <div v-if="form.auth_type === 'oauth_manual'" class="mt-4 space-y-4 p-4 bg-gray-900/50 rounded-lg">
            <div>
              <label class="block text-sm font-medium mb-2">Authorization URL</label>
              <input v-model="form.oauth_authorization_url" type="url" class="input" placeholder="https://provider.com/oauth/authorize" />
            </div>
            <div>
              <label class="block text-sm font-medium mb-2">Token URL</label>
              <input v-model="form.oauth_token_url" type="url" class="input" placeholder="https://provider.com/oauth/token" />
            </div>
            <div>
              <label class="block text-sm font-medium mb-2">Client ID</label>
              <input v-model="form.oauth_client_id" type="text" class="input" placeholder="your-client-id" />
            </div>
            <div>
              <label class="block text-sm font-medium mb-2">Client Secret (optional)</label>
              <input v-model="form.oauth_client_secret" type="password" class="input" placeholder="your-client-secret" />
            </div>
            <div>
              <label class="block text-sm font-medium mb-2">Scopes</label>
              <input v-model="form.oauth_scopes" type="text" class="input" placeholder="read write" />
            </div>
            <label class="flex items-center gap-2 text-sm">
              <input type="checkbox" v-model="form.oauth_use_pkce" class="text-indigo-500" />
              <span>Use PKCE (recommended)</span>
              <InfoTooltip 
                title="PKCE (Proof Key for Code Exchange)"
                content="PKCE adds extra security to the OAuth flow by using a cryptographic challenge. Required by OAuth 2.1 spec. Only disable if the server doesn't support it."
                type="warning"
              />
            </label>
          </div>
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
    
    <!-- Setup Modal -->
    <ServerSetupModal
      :is-open="showSetupModal"
      :server-name="form.name"
      :server-url="form.url"
      :auth-type="form.auth_type"
      :form-data="form"
      @close="showSetupModal = false"
      @complete="handleSetupComplete"
    />
  </DashboardLayout>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import InfoTooltip from '@/components/ui/InfoTooltip.vue'
import ServerSetupModal from '@/components/ui/ServerSetupModal.vue'
import { useServersStore, type TestResult } from '@/stores/servers'
import { 
  ExclamationCircleIcon, 
  CheckCircleIcon, 
  ExclamationTriangleIcon, 
  SignalIcon, 
  PlusIcon,
  LockClosedIcon,
  SparklesIcon
} from '@heroicons/vue/24/outline'

const router = useRouter()
const serversStore = useServersStore()

const form = reactive({
  name: '',
  url: '',
  transport: 'auto',
  auth_type: 'none',
  api_key: '',
  bearer_token: '',
  oauth_authorization_url: '',
  oauth_token_url: '',
  oauth_client_id: '',
  oauth_client_secret: '',
  oauth_scopes: '',
  oauth_use_pkce: true,
})

const error = ref<string | null>(null)
const testResult = ref<TestResult | null>(null)
const testing = ref(false)
const submitting = ref(false)
const showSetupModal = ref(false)

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
  // Open the setup modal instead of direct submission
  showSetupModal.value = true
}

function handleSetupComplete(serverName: string) {
  showSetupModal.value = false
  router.push(`/servers/${serverName}`)
}
</script>
