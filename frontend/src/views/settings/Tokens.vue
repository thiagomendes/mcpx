<template>
  <DashboardLayout>
    <div>
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-2xl font-bold mb-1">Personal Access Tokens</h1>
          <p class="text-gray-400">Create tokens to authenticate CLI tools and automations</p>
        </div>
        <button 
          @click="showCreateModal = true"
          :disabled="limitReached"
          :class="['btn flex items-center gap-2', limitReached ? 'btn-secondary opacity-50 cursor-not-allowed' : 'btn-primary']"
        >
          <PlusIcon class="w-4 h-4" />
          New Token
        </button>
      </div>

      <!-- Limit Warning (only for admins) -->
      <div v-if="limitReached" class="card bg-amber-500/20 border-amber-500/50 mb-4">
        <div class="flex items-start gap-3">
          <NoSymbolIcon class="w-5 h-5 text-amber-400 shrink-0" />
          <p class="text-gray-300 text-sm">
            You have {{ limitInfo.current }}/{{ limitInfo.max }} tokens. Delete one or increase the limit in 
            <router-link to="/settings/configuration" class="text-violet-400 hover:underline">Configuration</router-link>.
          </p>
        </div>
      </div>

      <!-- Token List -->
      <div class="card">
        <div v-if="loading" class="py-8 text-center text-gray-400">
          Loading tokens...
        </div>
        
        <div v-else-if="tokens.length === 0" class="py-8 text-center">
          <KeyIcon class="w-12 h-12 mx-auto text-gray-600 mb-3" />
          <p class="text-gray-400">No tokens yet</p>
          <p class="text-sm text-gray-500">Create a token to authenticate your tools</p>
        </div>

        <div v-else class="divide-y divide-gray-700">
          <div 
            v-for="token in tokens" 
            :key="token.id"
            class="flex items-center justify-between py-4 first:pt-0 last:pb-0"
          >
            <div class="flex items-center gap-4">
              <div class="w-10 h-10 rounded-lg bg-violet-500/20 flex items-center justify-center">
                <KeyIcon class="w-5 h-5 text-violet-400" />
              </div>
              <div>
                <div class="font-medium">{{ token.name }}</div>
                <div class="text-sm text-gray-400 font-mono">{{ token.token_prefix }}...</div>
              </div>
            </div>
            <div class="flex items-center gap-4">
              <div class="text-right text-sm">
                <div v-if="token.last_used_at" class="text-gray-400">
                  Last used {{ formatDate(token.last_used_at) }}
                </div>
                <div v-else class="text-gray-500">Never used</div>
                <div v-if="token.expires_at" class="text-xs" :class="isExpired(token.expires_at) ? 'text-red-400' : 'text-gray-500'">
                  {{ isExpired(token.expires_at) ? 'Expired' : 'Expires' }} {{ formatDate(token.expires_at) }}
                </div>
              </div>
              <button 
                @click="revokeToken(token)"
                class="p-2 text-gray-500 hover:text-red-400 transition-colors"
                title="Revoke token"
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
        <h2 class="text-lg font-semibold mb-4">Create Personal Access Token</h2>
        
        <div v-if="!newToken">
          <form @submit.prevent="createToken">
            <div class="mb-4">
              <label class="block text-sm font-medium text-gray-400 mb-1">Token Name</label>
              <input 
                v-model="createForm.name"
                type="text" 
                class="input w-full"
                placeholder="e.g. Claude Desktop"
                required
                autofocus
              />
              <p class="text-xs text-gray-500 mt-1">A memorable name to identify this token</p>
            </div>

            <div class="mb-6">
              <label class="block text-sm font-medium text-gray-400 mb-1">Expiration</label>
              <select v-model="createForm.expires_in_days" class="input w-full">
                <option :value="null">Never expires</option>
                <option :value="30">30 days</option>
                <option :value="90">90 days</option>
                <option :value="365">1 year</option>
              </select>
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
                <span v-else>Create Token</span>
              </button>
            </div>
          </form>
        </div>

        <!-- Token Created Success -->
        <div v-else class="text-center">
          <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-green-500/20 flex items-center justify-center">
            <CheckIcon class="w-8 h-8 text-green-400" />
          </div>
          <h3 class="text-lg font-semibold mb-2">Token Created!</h3>
          <p class="text-gray-400 text-sm mb-4">
            Copy this token now. You won't be able to see it again!
          </p>

          <div class="mb-6 text-left">
            <label class="block text-xs font-medium text-gray-500 mb-1 uppercase">Your Token</label>
            <div class="flex items-center gap-2">
              <input 
                readonly 
                :value="newToken"
                class="input w-full text-sm font-mono bg-gray-800"
                @click="($event.target as HTMLInputElement).select()"
              />
              <button 
                @click="copyToken"
                class="btn btn-secondary p-2"
                title="Copy Token"
              >
                <ClipboardDocumentIcon v-if="!copied" class="w-5 h-5" />
                <CheckIcon v-else class="w-5 h-5 text-green-400" />
              </button>
            </div>
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
  KeyIcon, 
  TrashIcon, 
  ArrowLeftIcon,
  CheckIcon,
  ClipboardDocumentIcon,
  NoSymbolIcon
} from '@heroicons/vue/24/outline'
import api from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import { usePermissions } from '@/composables/usePermissions'

interface Token {
  id: string
  name: string
  token_prefix: string
  scopes: string[]
  last_used_at: string | null
  expires_at: string | null
  created_at: string
}

const authStore = useAuthStore()
usePermissions() // For permission checks if needed
const tokens = ref<Token[]>([])
const loading = ref(true)
const showCreateModal = ref(false)
const creating = ref(false)
const createError = ref<string | null>(null)
const createForm = ref({
  name: '',
  expires_in_days: null as number | null
})
const newToken = ref<string | null>(null)
const copied = ref(false)
const limitReached = ref(false)
const limitInfo = ref({ current: 0, max: 0 })

async function checkLimits() {
  try {
    const response = await fetch('/api/limits', {
      headers: { 'Authorization': `Bearer ${authStore.token}` }
    })
    if (response.ok) {
      const data = await response.json()
      limitInfo.value = { current: data.pats.current, max: data.pats.max }
      limitReached.value = !data.pats.can_create
    }
  } catch (e) {
    console.error('Failed to check limits:', e)
  }
}

async function loadTokens() {
  try {
    const response = await api.get<Token[]>('/tokens')
    tokens.value = response.data
  } catch (error) {
    console.error('Failed to load tokens:', error)
  } finally {
    loading.value = false
  }
}

async function createToken() {
  if (!createForm.value.name.trim()) return
  
  creating.value = true
  createError.value = null

  try {
    const response = await api.post('/tokens', {
      name: createForm.value.name.trim(),
      expires_in_days: createForm.value.expires_in_days
    })
    newToken.value = response.data.token
    await loadTokens()
  } catch (err: unknown) {
    const error_obj = err as { response?: { data?: { message?: string } } }
    console.error('Failed to create token:', err)
    createError.value = error_obj.response?.data?.message || 'Failed to create token'
  } finally {
    creating.value = false
  }
}

async function revokeToken(token: Token) {
  if (!confirm(`Are you sure you want to revoke the token "${token.name}"? This cannot be undone.`)) {
    return
  }
  
  try {
    await api.delete(`/tokens/${token.id}`)
    await loadTokens()
  } catch (error) {
    console.error('Failed to revoke token:', error)
    alert('Failed to revoke token')
  }
}

function closeCreateModal() {
  showCreateModal.value = false
  newToken.value = null
  createForm.value = { name: '', expires_in_days: null }
}

function copyToken() {
  if (newToken.value) {
    navigator.clipboard.writeText(newToken.value)
    copied.value = true
    setTimeout(() => copied.value = false, 2000)
  }
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString()
}

function isExpired(dateStr: string) {
  return new Date(dateStr) < new Date()
}

onMounted(() => {
  loadTokens()
  checkLimits()
})
</script>
