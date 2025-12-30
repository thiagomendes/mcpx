<template>
  <DashboardLayout>
    <div class="max-w-4xl">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-2xl font-bold mb-1">Service Accounts</h1>
          <p class="text-gray-400">Create service accounts for automated systems and CI/CD pipelines</p>
        </div>
        <button 
          @click="showCreateModal = true"
          class="btn btn-primary flex items-center gap-2"
        >
          <PlusIcon class="w-4 h-4" />
          New Service Account
        </button>
      </div>

      <!-- Service Account List -->
      <div class="card">
        <div v-if="loading" class="py-8 text-center text-gray-400">
          Loading service accounts...
        </div>
        
        <div v-else-if="accounts.length === 0" class="py-8 text-center">
          <CogIcon class="w-12 h-12 mx-auto text-gray-600 mb-3" />
          <p class="text-gray-400">No service accounts yet</p>
          <p class="text-sm text-gray-500">Create a service account for M2M authentication</p>
        </div>

        <div v-else class="divide-y divide-gray-700">
          <div 
            v-for="account in accounts" 
            :key="account.id"
            class="flex items-center justify-between py-4 first:pt-0 last:pb-0"
          >
            <div class="flex items-center gap-4">
              <div
                class="w-10 h-10 rounded-lg flex items-center justify-center"
                :class="account.enabled ? 'bg-emerald-500/20' : 'bg-gray-500/20'"
              >
                <CogIcon class="w-5 h-5" :class="account.enabled ? 'text-emerald-400' : 'text-gray-500'" />
              </div>
              <div>
                <div class="font-medium flex items-center gap-2">
                  {{ account.name }}
                  <span v-if="!account.enabled" class="text-xs px-2 py-0.5 bg-gray-600 text-gray-300 rounded">
                    Disabled
                  </span>
                </div>
                <div class="text-sm text-gray-400 font-mono">{{ account.client_id }}</div>
              </div>
            </div>
            <div class="flex items-center gap-4">
              <div class="text-right text-sm">
                <div v-if="account.last_used_at" class="text-gray-400">
                  Last used {{ formatDate(account.last_used_at) }}
                </div>
                <div v-else class="text-gray-500">Never used</div>
                <div class="text-xs text-gray-500">
                  {{ formatScopes(account.scopes) }}
                </div>
              </div>
              <button 
                @click="toggleEnabled(account)"
                class="p-2 transition-colors"
                :class="account.enabled ? 'text-gray-500 hover:text-yellow-400' : 'text-gray-500 hover:text-green-400'"
                :title="account.enabled ? 'Disable' : 'Enable'"
              >
                <PauseCircleIcon v-if="account.enabled" class="w-5 h-5" />
                <PlayCircleIcon v-else class="w-5 h-5" />
              </button>
              <button 
                @click="deleteAccount(account)"
                class="p-2 text-gray-500 hover:text-red-400 transition-colors"
                title="Delete service account"
              >
                <TrashIcon class="w-5 h-5" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Back link -->
      <div class="mt-6">
        <router-link 
          to="/settings"
          class="text-gray-400 hover:text-white text-sm flex items-center gap-1"
        >
          <ArrowLeftIcon class="w-4 h-4" />
          Back to Settings
        </router-link>
      </div>
    </div>

    <!-- Create Modal -->
    <div 
      v-if="showCreateModal" 
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      @click.self="showCreateModal = false"
    >
      <div class="card w-full max-w-md">
        <h2 class="text-lg font-semibold mb-4">Create Service Account</h2>
        
        <div v-if="!newCredentials">
          <form @submit.prevent="createAccount">
            <div class="mb-4">
              <label class="block text-sm font-medium text-gray-400 mb-1">Name</label>
              <input 
                v-model="createForm.name"
                type="text" 
                class="input w-full"
                placeholder="e.g. CI/CD Pipeline"
                required
                autofocus
              />
              <p class="text-xs text-gray-500 mt-1">A name to identify this service account</p>
            </div>

            <div class="mb-4">
              <label class="block text-sm font-medium text-gray-400 mb-1">Description</label>
              <input 
                v-model="createForm.description"
                type="text" 
                class="input w-full"
                placeholder="e.g. Automated deployment pipeline"
              />
            </div>

            <div class="mb-6">
              <label class="block text-sm font-medium text-gray-400 mb-2">Permissions</label>
              <div class="space-y-3">
                <label
                  class="flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors"
                  :class="createForm.accessLevel === 'full' ? 'border-emerald-500 bg-emerald-500/10' : 'border-gray-700 hover:border-gray-600'"
                >
                  <input type="radio" v-model="createForm.accessLevel" value="full" class="mt-1" />
                  <div>
                    <div class="font-medium">Full Access</div>
                    <div class="text-sm text-gray-500">Can list and execute tools</div>
                  </div>
                </label>
                <label
                  class="flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors"
                  :class="createForm.accessLevel === 'read' ? 'border-emerald-500 bg-emerald-500/10' : 'border-gray-700 hover:border-gray-600'"
                >
                  <input type="radio" v-model="createForm.accessLevel" value="read" class="mt-1" />
                  <div>
                    <div class="font-medium">Read Only</div>
                    <div class="text-sm text-gray-500">Can only list tools and resources</div>
                  </div>
                </label>
              </div>
            </div>

            <div v-if="createError" class="mb-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-sm">
              {{ createError }}
            </div>

            <div class="flex items-center gap-3">
              <button 
                type="button" 
                @click="showCreateModal = false"
                class="btn btn-secondary flex-1"
                :disabled="creating"
              >
                Cancel
              </button>
              <button 
                type="submit" 
                class="btn btn-primary flex-1"
                :disabled="creating || !createForm.name.trim()"
              >
                <span v-if="creating">Creating...</span>
                <span v-else>Create</span>
              </button>
            </div>
          </form>
        </div>

        <!-- Credentials Created Success -->
        <div v-else class="text-center">
          <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-green-500/20 flex items-center justify-center">
            <CheckIcon class="w-8 h-8 text-green-400" />
          </div>
          <h3 class="text-lg font-semibold mb-2">Service Account Created!</h3>
          <p class="text-gray-400 text-sm mb-4">
            Copy these credentials now. The secret won't be shown again!
          </p>

          <div class="mb-4 text-left">
            <label class="block text-xs font-medium text-gray-500 mb-1 uppercase">Client ID</label>
            <div class="flex items-center gap-2">
              <input 
                readonly 
                :value="newCredentials.client_id"
                class="input w-full text-sm font-mono bg-gray-800"
                @click="($event.target as HTMLInputElement).select()"
              />
              <button 
                @click="copyToClipboard(newCredentials.client_id)"
                class="btn btn-secondary p-2"
                title="Copy"
              >
                <ClipboardDocumentIcon class="w-5 h-5" />
              </button>
            </div>
          </div>

          <div class="mb-6 text-left">
            <label class="block text-xs font-medium text-gray-500 mb-1 uppercase">Client Secret</label>
            <div class="flex items-center gap-2">
              <input 
                readonly 
                :value="newCredentials.client_secret"
                class="input w-full text-sm font-mono bg-gray-800"
                @click="($event.target as HTMLInputElement).select()"
              />
              <button 
                @click="copyToClipboard(newCredentials.client_secret)"
                class="btn btn-secondary p-2"
                title="Copy"
              >
                <ClipboardDocumentIcon class="w-5 h-5" />
              </button>
            </div>
          </div>

          <div class="mb-6 text-left p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg">
            <p class="text-sm text-blue-400 font-medium mb-1">Usage:</p>
            <pre class="text-xs text-gray-400 overflow-x-auto">curl -X POST {{ apiUrl }}/api/auth/token \
  -d "grant_type=client_credentials&amp;client_id={{ newCredentials.client_id }}&amp;client_secret=..."</pre>
          </div>

          <button 
            @click="closeCreateModal"
            class="btn btn-primary w-full"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { 
  PlusIcon, 
  CogIcon, 
  TrashIcon, 
  ArrowLeftIcon,
  CheckIcon,
  ClipboardDocumentIcon,
  PauseCircleIcon,
  PlayCircleIcon
} from '@heroicons/vue/24/outline'
import api from '@/api/client'

interface ServiceAccount {
  id: string
  name: string
  description: string | null
  client_id: string
  scopes: string[]
  enabled: boolean
  last_used_at: string | null
  created_at: string
}

interface NewCredentials {
  id: string
  name: string
  client_id: string
  client_secret: string
  scopes: string[]
}

const apiUrl = window.location.origin.replace(':3000', ':8080')

const accounts = ref<ServiceAccount[]>([])
const loading = ref(true)
const showCreateModal = ref(false)
const creating = ref(false)
const createError = ref<string | null>(null)
const createForm = ref({
  name: '',
  description: '',
  accessLevel: 'full' as 'full' | 'read'
})
const newCredentials = ref<NewCredentials | null>(null)

async function loadAccounts() {
  try {
    const response = await api.get<ServiceAccount[]>('/service-accounts')
    accounts.value = response.data
  } catch (error) {
    console.error('Failed to load service accounts:', error)
  } finally {
    loading.value = false
  }
}

async function createAccount() {
  if (!createForm.value.name.trim()) return
  
  creating.value = true
  createError.value = null

  try {
    // Convert accessLevel to scopes
    const scopes = createForm.value.accessLevel === 'full'
      ? ['mcp:server:read', 'mcp:tool:execute']
      : ['mcp:server:read']
    
    const response = await api.post<NewCredentials>('/service-accounts', {
      name: createForm.value.name.trim(),
      description: createForm.value.description || null,
      scopes
    })
    newCredentials.value = response.data
    await loadAccounts()
  } catch (err: unknown) {
    const error_obj = err as { response?: { data?: string } }
    console.error('Failed to create service account:', err)
    createError.value = error_obj.response?.data || 'Failed to create service account'
  } finally {
    creating.value = false
  }
}

async function toggleEnabled(account: ServiceAccount) {
  try {
    await api.put(`/service-accounts/${account.id}`, {
      enabled: !account.enabled
    })
    await loadAccounts()
  } catch (error) {
    console.error('Failed to update service account:', error)
    alert('Failed to update service account')
  }
}

async function deleteAccount(account: ServiceAccount) {
  if (!confirm(`Are you sure you want to delete "${account.name}"? This cannot be undone.`)) {
    return
  }
  
  try {
    await api.delete(`/service-accounts/${account.id}`)
    await loadAccounts()
  } catch (error) {
    console.error('Failed to delete service account:', error)
    alert('Failed to delete service account')
  }
}

function closeCreateModal() {
  showCreateModal.value = false
  newCredentials.value = null
  createForm.value = { name: '', description: '', accessLevel: 'full' }
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text)
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString()
}

function formatScopes(scopes: string[]) {
  return scopes.join(', ')
}

onMounted(() => {
  loadAccounts()
})
</script>
