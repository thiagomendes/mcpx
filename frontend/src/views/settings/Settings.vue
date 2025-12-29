<template>
  <DashboardLayout>
    <div class="w-full">
      <h1 class="text-2xl font-bold mb-2">Organization Settings</h1>
      <p class="text-gray-400 mb-8">Manage your organization settings and team members.</p>

      <!-- Org Info Card -->
      <div class="card mb-8">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-violet-500/30 to-purple-600/30 flex items-center justify-center">
              <span class="text-xl font-bold text-violet-300">{{ currentOrg?.name?.charAt(0)?.toUpperCase() }}</span>
            </div>
            <div>
              <h2 class="text-lg font-semibold">{{ currentOrg?.name }}</h2>
              <p class="text-sm text-gray-400 font-mono">{{ currentOrg?.slug }}</p>
            </div>
          </div>
          <span 
            class="px-3 py-1 text-xs font-medium rounded-full"
            :class="currentOrg?.is_personal ? 'bg-blue-500/20 text-blue-400' : 'bg-violet-500/20 text-violet-400'"
          >
            {{ currentOrg?.is_personal ? 'Personal' : 'Organization' }}
          </span>
        </div>
      </div>

      <!-- Quick Links Section -->
      <div class="mb-8">
        <h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-4">Settings</h2>
        <div class="space-y-4">
          <!-- Team Members - only for team orgs -->
          <router-link 
            v-if="!currentOrg?.is_personal"
            to="/settings/members"
            class="card hover:border-violet-500/50 transition-all group flex items-center gap-4"
          >
            <div class="w-12 h-12 rounded-xl bg-violet-500/20 flex items-center justify-center shrink-0">
              <UsersIcon class="w-6 h-6 text-violet-400" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="font-medium group-hover:text-violet-400 transition-colors">Team Members</h3>
              <p class="text-sm text-gray-400">Manage who has access to this organization</p>
            </div>
            <ChevronRightIcon class="w-5 h-5 text-gray-500 shrink-0" />
          </router-link>

          <!-- Access Tokens - always visible -->
          <router-link 
            to="/settings/tokens"
            class="card hover:border-amber-500/50 transition-all group flex items-center gap-4"
          >
            <div class="w-12 h-12 rounded-xl bg-amber-500/20 flex items-center justify-center shrink-0">
              <KeyIcon class="w-6 h-6 text-amber-400" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="font-medium group-hover:text-amber-400 transition-colors">Access Tokens</h3>
              <p class="text-sm text-gray-400">Create tokens for CLI tools and automations</p>
            </div>
            <ChevronRightIcon class="w-5 h-5 text-gray-500 shrink-0" />
          </router-link>

          <!-- Service Accounts (M2M) - visible to admins -->
          <router-link 
            v-if="authStore.canAdmin"
            to="/settings/service-accounts"
            class="card hover:border-emerald-500/50 transition-all group flex items-center gap-4"
          >
            <div class="w-12 h-12 rounded-xl bg-emerald-500/20 flex items-center justify-center shrink-0">
              <CogIcon class="w-6 h-6 text-emerald-400" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="font-medium group-hover:text-emerald-400 transition-colors">Service Accounts</h3>
              <p class="text-sm text-gray-400">Create M2M credentials for automated systems</p>
            </div>
            <ChevronRightIcon class="w-5 h-5 text-gray-500 shrink-0" />
          </router-link>
        </div>
      </div>

      <!-- Linked Identities Section -->
      <div class="mb-8">
        <h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-4">Linked Accounts</h2>
        <div class="card">
          <div class="divide-y divide-gray-700/50">
            <div 
              v-for="identity in identities" 
              :key="identity.provider"
              class="flex items-center justify-between py-3 first:pt-0 last:pb-0"
            >
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-gray-700/50 flex items-center justify-center">
                  <component :is="getProviderIcon(identity.provider)" class="w-5 h-5" />
                </div>
                <div>
                  <div class="font-medium capitalize flex items-center gap-2">
                    {{ identity.provider }}
                    <span 
                      v-if="identity.provider === authStore.authProvider" 
                      class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-green-500/20 text-green-400 border border-green-500/30 uppercase tracking-wide"
                    >
                      Current
                    </span>
                  </div>
                </div>
              </div>
              <span class="text-sm text-gray-500">{{ formatDate(identity.created_at) }}</span>
            </div>
            <div v-if="identities.length === 0" class="text-gray-400 text-sm py-3">
              No linked accounts
            </div>
          </div>
        </div>
      </div>

      <!-- Danger Zone Section - Team Orgs -->
      <div v-if="!currentOrg?.is_personal && isOwner" class="mb-8">
        <h2 class="text-sm font-semibold text-red-400/70 uppercase tracking-wider mb-4">Danger Zone</h2>
        <div class="card border-red-500/20 bg-red-500/5">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
              <div class="w-10 h-10 rounded-lg bg-red-500/10 flex items-center justify-center">
                <TrashIcon class="w-5 h-5 text-red-400" />
              </div>
              <div>
                <h3 class="font-medium text-red-400">Delete Organization</h3>
                <p class="text-sm text-gray-400">Permanently delete this organization and all its data</p>
              </div>
            </div>
            <button 
              @click="showDeleteModal = true"
              class="px-4 py-2 text-sm font-medium bg-red-500/10 text-red-400 hover:bg-red-500/20 rounded-lg transition-colors border border-red-500/20"
            >
              Delete
            </button>
          </div>
        </div>
      </div>

      <!-- Danger Zone Section - Personal Account -->
      <div v-if="currentOrg?.is_personal" class="mb-8">
        <h2 class="text-sm font-semibold text-red-400/70 uppercase tracking-wider mb-4">Danger Zone</h2>
        <div class="card border-red-500/20 bg-red-500/5">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
              <div class="w-10 h-10 rounded-lg bg-red-500/10 flex items-center justify-center">
                <ExclamationTriangleIcon class="w-5 h-5 text-red-400" />
              </div>
              <div>
                <h3 class="font-medium text-red-400">Delete Account</h3>
                <p class="text-sm text-gray-400">Permanently delete your account and all personal data</p>
              </div>
            </div>
            <button 
              @click="openDeleteAccountModal"
              class="px-4 py-2 text-sm font-medium bg-red-500/10 text-red-400 hover:bg-red-500/20 rounded-lg transition-colors border border-red-500/20"
            >
              Delete Account
            </button>
          </div>
        </div>
      </div>
    </div>


    <!-- Delete Modal -->
    <div 
      v-if="showDeleteModal" 
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      @click.self="showDeleteModal = false"
    >
      <div class="card w-full max-w-md border-red-500/20">
        <h2 class="text-lg font-semibold mb-2 text-red-400">Delete Organization?</h2>
        <p class="text-gray-400 text-sm mb-4">
          This action cannot be undone. This will permanently delete 
          <span class="font-bold text-white">{{ currentOrg?.name }}</span> 
          and remove all associated data and member access.
        </p>
        
        <div class="mb-6">
          <label class="block text-xs font-medium text-gray-500 mb-1 uppercase">
            Type <span class="text-white select-none">{{ currentOrg?.name }}</span> to confirm
          </label>
          <input 
            v-model="deleteConfirmation"
            type="text" 
            class="input w-full border-red-500/20 focus:border-red-500"
            :placeholder="currentOrg?.name"
          />
        </div>

        <div class="flex items-center gap-3">
          <button 
            @click="showDeleteModal = false"
            class="btn btn-secondary flex-1"
            :disabled="deleteLoading"
          >
            Cancel
          </button>
          <button 
            @click="handleDeleteOrg"
            class="btn bg-red-600 hover:bg-red-700 text-white border-none flex-1"
            :disabled="deleteLoading || deleteConfirmation !== currentOrg?.name"
          >
            <span v-if="deleteLoading">Deleting...</span>
            <span v-else>Delete Organization</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Delete Account Modal -->
    <div 
      v-if="showDeleteAccountModal" 
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      @click.self="showDeleteAccountModal = false"
    >
      <div class="card w-full max-w-md border-red-500/20">
        <h2 class="text-lg font-semibold mb-2 text-red-400">Delete Your Account?</h2>
        
        <!-- Loading state -->
        <div v-if="previewLoading" class="text-gray-400 text-sm py-4 text-center">
          Loading...
        </div>
        
        <!-- Orgs to delete warning -->
        <div v-else>
          <p class="text-gray-400 text-sm mb-4">
            This action <span class="font-bold text-red-400">cannot be undone</span>. The following will be permanently deleted:
          </p>
          
          <ul class="text-sm text-gray-400 mb-4 list-disc list-inside space-y-1">
            <li>Your account and profile</li>
            <li>Your membership in all organizations</li>
          </ul>

          <!-- Orgs that will be deleted -->
          <div v-if="deletionPreview?.orgs_to_delete?.length" class="mb-4">
            <p class="text-sm font-medium text-red-400 mb-2">
              Organizations you own (will be deleted):
            </p>
            <div class="space-y-2">
              <div 
                v-for="org in deletionPreview.orgs_to_delete" 
                :key="org.id"
                class="flex items-center gap-2 text-sm bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2"
              >
                <TrashIcon class="w-4 h-4 text-red-400 shrink-0" />
                <span class="text-white">{{ org.name }}</span>
                <span v-if="org.is_personal" class="text-xs text-gray-500">(Personal)</span>
              </div>
            </div>
            <p class="text-xs text-gray-500 mt-2">
              All servers, gateways, and tokens in these organizations will be deleted.
            </p>
          </div>
        </div>
        
        <div class="mb-6">
          <label class="block text-xs font-medium text-gray-500 mb-1 uppercase">
            Type <span class="text-white select-none">DELETE</span> to confirm
          </label>
          <input 
            v-model="deleteAccountConfirmation"
            type="text" 
            class="input w-full border-red-500/20 focus:border-red-500"
            placeholder="DELETE"
          />
        </div>

        <div class="flex items-center gap-3">
          <button 
            @click="showDeleteAccountModal = false"
            class="btn btn-secondary flex-1"
            :disabled="deleteAccountLoading"
          >
            Cancel
          </button>
          <button 
            @click="handleDeleteAccount"
            class="btn bg-red-600 hover:bg-red-700 text-white border-none flex-1"
            :disabled="deleteAccountLoading || deleteAccountConfirmation !== 'DELETE'"
          >
            <span v-if="deleteAccountLoading">Deleting...</span>
            <span v-else>Delete My Account</span>
          </button>
        </div>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
// import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { UsersIcon, ChevronRightIcon, TrashIcon, KeyIcon, ExclamationTriangleIcon, CogIcon } from '@heroicons/vue/24/outline'
import GoogleIcon from '@/components/icons/GoogleIcon.vue'
import GitHubIcon from '@/components/icons/GitHubIcon.vue'
import MicrosoftIcon from '@/components/icons/MicrosoftIcon.vue'
import api from '@/api/client'

// const router = useRouter()
const authStore = useAuthStore()

// Types
interface OrgToDelete {
  id: string
  name: string
  is_personal: boolean
}

interface DeletionPreview {
  user_email: string
  orgs_to_delete: OrgToDelete[]
}

// State
const showDeleteModal = ref(false)
const deleteConfirmation = ref('')
const deleteLoading = ref(false)

const showDeleteAccountModal = ref(false)
const deleteAccountConfirmation = ref('')
const deleteAccountLoading = ref(false)
const deletionPreview = ref<DeletionPreview | null>(null)
const previewLoading = ref(false)

const currentOrg = computed(() => authStore.currentOrg)
const identities = computed(() => authStore.identities)
const isOwner = computed(() => currentOrg.value?.role === 'owner')

// Fetch preview when opening delete account modal
async function openDeleteAccountModal() {
  showDeleteAccountModal.value = true
  previewLoading.value = true
  try {
    const response = await api.get('/account/deletion-preview')
    deletionPreview.value = response.data
  } catch (error) {
    console.error('Failed to fetch deletion preview:', error)
  } finally {
    previewLoading.value = false
  }
}

async function handleDeleteOrg() {
  if (!currentOrg.value || deleteConfirmation.value !== currentOrg.value.name) return

  deleteLoading.value = true
  try {
    await api.delete(`/orgs/${currentOrg.value.id}`)
    
    // Refresh user data (switch to remaining org)
    await authStore.fetchUser()
    
    // Redirect to dashboard (will reload with new context)
    window.location.href = '/dashboard'
  } catch (error) {
    console.error('Failed to delete org:', error)
    alert('Failed to delete organization. Please try again.')
    deleteLoading.value = false
  }
}

async function handleDeleteAccount() {
  if (deleteAccountConfirmation.value !== 'DELETE') return

  deleteAccountLoading.value = true
  try {
    await api.delete('/account')
    
    // Clear auth state and redirect to account-deleted page
    authStore.logout()
    window.location.href = '/account-deleted'
  } catch (error) {
    console.error('Failed to delete account:', error)
    alert('Failed to delete account. Please try again.')
    deleteAccountLoading.value = false
  }
}

function getProviderIcon(provider: string) {
  switch (provider) {
    case 'google': return GoogleIcon
    case 'github': return GitHubIcon
    case 'microsoft': return MicrosoftIcon
    default: return 'span'
  }
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString()
}
</script>
