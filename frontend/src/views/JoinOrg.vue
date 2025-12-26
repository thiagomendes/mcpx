<template>
  <div class="min-h-screen bg-black text-white flex items-center justify-center p-4">
    <div class="card w-full max-w-md text-center p-8">
      <div v-if="loading">
        <div class="w-12 h-12 border-4 border-violet-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
        <h2 class="text-xl font-bold mb-2">Verifying Invite...</h2>
        <p class="text-gray-400">Please wait.</p>
      </div>

      <div v-else-if="error">
        <div class="w-16 h-16 bg-red-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
          <XMarkIcon class="w-8 h-8 text-red-500" />
        </div>
        <h2 class="text-xl font-bold mb-2 text-red-400">Failed to Join</h2>
        <p class="text-gray-400 mb-6">{{ error }}</p>
        <router-link to="/dashboard" class="btn btn-secondary w-full">
          Go to Dashboard
        </router-link>
      </div>

      <div v-else-if="step === 'confirm' && details">
        <div class="w-16 h-16 bg-violet-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
          <EnvelopeOpenIcon class="w-8 h-8 text-violet-400" />
        </div>
        <h2 class="text-xl font-bold mb-2">You've been invited!</h2>
        
        <div class="mb-6 p-4 bg-gray-800 rounded-lg text-left">
          <div class="text-sm text-gray-400 mb-1">Organization</div>
          <div class="font-bold text-lg mb-3">{{ details.org_name }}</div>
          
          <div class="text-sm text-gray-400 mb-1">Invited by</div>
          <div class="font-medium">
            {{ details.inviter_name || details.inviter_email || 'Unknown User' }}
          </div>
          <div class="text-xs text-gray-500">{{ details.inviter_email }}</div>
        </div>

        <div class="flex gap-3">
          <router-link to="/dashboard" class="btn btn-secondary flex-1">
            Cancel
          </router-link>
          <button @click="acceptInvite" class="btn btn-primary flex-1" :disabled="joining">
            <span v-if="joining">Joining...</span>
            <span v-else>Accept Invite</span>
          </button>
        </div>
      </div>

      <div v-else-if="step === 'success'">
        <div class="w-16 h-16 bg-green-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
          <CheckIcon class="w-8 h-8 text-green-500" />
        </div>
        <h2 class="text-xl font-bold mb-2 text-green-400">Welcome!</h2>
        <p class="text-gray-400 mb-6">You have joined <strong>{{ details?.org_name }}</strong>.</p>
        
        <div class="animate-pulse text-sm text-gray-500">Redirecting to settings...</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import api from '@/api/client'
import { CheckIcon, XMarkIcon, EnvelopeOpenIcon } from '@heroicons/vue/24/outline'

interface InviteDetails {
  email: string
  org_id: string
  org_name: string
  role: string
  inviter_name?: string
  inviter_email?: string
}

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const loading = ref(true)
const joining = ref(false)
const error = ref<string | null>(null)
const step = ref<'confirm' | 'success'>('confirm')
const token = ref('')
const details = ref<InviteDetails | null>(null)

onMounted(async () => {
  token.value = route.params.token as string
  if (!token.value) {
    error.value = 'Invalid invite link'
    loading.value = false
    return
  }

  // Ensure user is logged in
  if (!authStore.isAuthenticated) {
    localStorage.setItem('auth_redirect', route.fullPath)
    router.push('/login')
    return
  }
  
  // Fetch details
  try {
    const response = await api.get<InviteDetails>(`/orgs/join/${token.value}`)
    details.value = response.data
    loading.value = false
  } catch (err: unknown) {
    const error_obj = err as { response?: { data?: { message?: string } } }
    console.error('Fetch invite error:', err)
    error.value = error_obj.response?.data?.message || 'Failed to load invite details'
    loading.value = false
  }
})

async function acceptInvite() {
  joining.value = true
  try {
    await api.post(`/orgs/join/${token.value}`)
    
    // Refresh user data (this gets the new org list)
    await authStore.fetchUser()
    
    // Switch context to the new org!
    if (details.value?.org_id) {
        authStore.setCurrentOrg(details.value.org_id)
        // We can also trigger the switch API if needed, but managing local state might be enough 
        // if user data refresh already verified membership. 
        // Ideally we call switchOrg but that requires a backend call to Mint a new token.
        await authStore.switchOrg(details.value.org_id)
    }

    step.value = 'success'
    
    setTimeout(() => {
        router.push('/settings')
    }, 1500)
    
  } catch (err: unknown) {
    const error_obj = err as { response?: { data?: { message?: string }; status?: number } }
    console.error('Join error:', err)
    if (error_obj.response?.status === 409) {
        step.value = 'success'
        setTimeout(() => router.push('/settings'), 1500)
    } else {
        error.value = error_obj.response?.data?.message || 'Failed to join organization'
    }
  } finally {
    joining.value = false
  }
}
</script>
