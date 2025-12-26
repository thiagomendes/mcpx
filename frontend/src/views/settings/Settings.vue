<template>
  <DashboardLayout>
    <div class="max-w-4xl">
      <h1 class="text-2xl font-bold mb-2">Organization Settings</h1>
      <p class="text-gray-400 mb-8">Manage your organization settings and team members.</p>

      <!-- Org Info Card -->
      <div class="card mb-6">
        <div class="flex items-start justify-between">
          <div>
            <h2 class="text-lg font-semibold mb-1">{{ currentOrg?.name }}</h2>
            <p class="text-sm text-gray-400">
              <span class="font-mono bg-gray-800 px-2 py-0.5 rounded">{{ currentOrg?.slug }}</span>
            </p>
          </div>
          <span 
            class="px-2 py-1 text-xs rounded-full"
            :class="currentOrg?.is_personal ? 'bg-blue-500/20 text-blue-400' : 'bg-violet-500/20 text-violet-400'"
          >
            {{ currentOrg?.is_personal ? 'Personal' : 'Organization' }}
          </span>
        </div>
      </div>

      <!-- Quick Links -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <router-link 
          v-if="!currentOrg?.is_personal"
          to="/settings/members"
          class="card hover:border-violet-500/50 transition-colors group"
        >
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 rounded-lg bg-violet-500/20 flex items-center justify-center">
              <UsersIcon class="w-6 h-6 text-violet-400" />
            </div>
            <div class="flex-1">
              <h3 class="font-medium group-hover:text-violet-400 transition-colors">Team Members</h3>
              <p class="text-sm text-gray-400">Manage who has access to this organization</p>
            </div>
            <ChevronRightIcon class="w-5 h-5 text-gray-500" />
          </div>
        </router-link>

        <div 
          v-if="!currentOrg?.is_personal && isOwner"
          class="card border-red-500/20 hover:border-red-500/40 transition-colors"
        >
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 rounded-lg bg-red-500/10 flex items-center justify-center">
              <TrashIcon class="w-6 h-6 text-red-400" />
            </div>
            <div class="flex-1">
              <h3 class="font-medium text-red-400">Danger Zone</h3>
              <p class="text-sm text-gray-400">Irreversibly delete this organization</p>
            </div>
            <button 
              @click="showDeleteModal = true"
              class="px-3 py-1.5 text-xs font-medium bg-red-500/10 text-red-400 hover:bg-red-500/20 rounded-lg transition-colors border border-red-500/20"
            >
              Delete Organization
            </button>
          </div>
        </div>
      </div>

      <!-- Linked Identities -->
      <div class="mt-8">
        <h2 class="text-lg font-semibold mb-4">Linked Accounts</h2>
        <div class="card">
          <div class="space-y-3">
            <div 
              v-for="identity in identities" 
              :key="identity.provider"
              class="flex items-center justify-between py-2"
            >
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-gray-700 flex items-center justify-center">
                  <component :is="getProviderIcon(identity.provider)" class="w-4 h-4" />
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
              <span class="text-sm text-gray-400">
                Linked {{ formatDate(identity.created_at) }}
              </span>
            </div>
            <div v-if="identities.length === 0" class="text-gray-400 text-sm py-2">
              No linked accounts
            </div>
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
  </DashboardLayout>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
// import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { UsersIcon, ChevronRightIcon, TrashIcon } from '@heroicons/vue/24/outline'
import GoogleIcon from '@/components/icons/GoogleIcon.vue'
import GitHubIcon from '@/components/icons/GitHubIcon.vue'
import MicrosoftIcon from '@/components/icons/MicrosoftIcon.vue'
import api from '@/api/client'

// const router = useRouter()
const authStore = useAuthStore()

// State
const showDeleteModal = ref(false)
const deleteConfirmation = ref('')
const deleteLoading = ref(false)

const currentOrg = computed(() => authStore.currentOrg)
const identities = computed(() => authStore.identities)
const isOwner = computed(() => currentOrg.value?.role === 'owner')

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
