<template>
  <div class="min-h-screen bg-background flex items-center justify-center">
    <div class="text-center">
      <div v-if="processing" class="text-gray-400">
        <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
        <p>Completing authorization...</p>
      </div>
      
      <div v-else-if="success" class="text-green-400">
        <CheckCircleIcon class="w-16 h-16 mx-auto mb-4" />
        <h1 class="text-xl font-bold mb-2">Authorization Successful!</h1>
        <p class="text-gray-400 mb-4">You can close this window.</p>
        <p class="text-sm text-gray-500">Closing automatically...</p>
      </div>
      
      <div v-else class="text-red-400">
        <ExclamationCircleIcon class="w-16 h-16 mx-auto mb-4" />
        <h1 class="text-xl font-bold mb-2">Authorization Failed</h1>
        <p class="text-gray-400 mb-4">{{ error }}</p>
        <button @click="closeWindow" class="btn btn-primary">Close Window</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { CheckCircleIcon, ExclamationCircleIcon } from '@heroicons/vue/24/outline'
import { handleOAuthCallback } from '@/lib/mcp'
import { McpxOAuthProvider } from '@/lib/mcp/oauthProvider'

const processing = ref(true)
const success = ref(false)
const error = ref<string | null>(null)

onMounted(async () => {
  const urlParams = new URLSearchParams(window.location.search)
  const code = urlParams.get('code')
  const state = urlParams.get('state')
  
  console.log('[OAuth Callback] code:', !!code, 'state:', state)
  
  if (!code) {
    processing.value = false
    error.value = 'Missing authorization code'
    return
  }
  
  if (!state) {
    processing.value = false
    error.value = 'Missing state parameter'
    return
  }
  
  // Extract serverName from state parameter
  // State format: mcpx:<serverName>:<random>
  const serverName = McpxOAuthProvider.parseServerNameFromState(state)
  console.log('[OAuth Callback] Parsed server name:', serverName)
  
  if (!serverName) {
    processing.value = false
    error.value = 'Invalid state parameter - could not determine server'
    return
  }
  
  try {
    const result = await handleOAuthCallback(serverName, code, state)
    processing.value = false
    
    if (result.success) {
      success.value = true
      // Close window after short delay
      setTimeout(() => {
        window.close()
      }, 2000)
    } else {
      error.value = result.error || 'Unknown error during token exchange'
    }
  } catch (e: any) {
    processing.value = false
    error.value = e.message || 'Failed to complete authorization'
  }
})

function closeWindow() {
  window.close()
}
</script>
