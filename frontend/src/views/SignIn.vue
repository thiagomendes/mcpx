<template>
  <div class="min-h-screen flex items-center justify-center">
    <div class="max-w-md w-full px-4">
      <div class="card text-center">
        <!-- Logo -->
        <div class="flex items-center justify-center gap-3 mb-8">
          <img src="@/assets/mcpx-logo.svg" alt="mcpx" class="w-12 h-12 object-contain" />
          <span class="text-2xl font-bold text-gradient">mcpx</span>
        </div>

        <h1 class="text-2xl font-bold mb-2">Sign In</h1>
        <p class="text-gray-400 mb-8">Sign in to manage your MCP servers</p>

        <!-- Identity Provider Buttons -->
        <div class="space-y-3">
          <button 
            v-for="provider in enabledProviders" 
            :key="provider.id"
            @click="login(provider.id)"
            class="w-full flex items-center justify-center gap-3 px-4 py-3 rounded-lg font-medium transition-colors hover:opacity-90"
            :style="{ backgroundColor: provider.color, color: '#fff' }"
          >
            <component :is="getProviderIcon(provider.id)" class="w-5 h-5" />
            Sign in with {{ provider.name }}
          </button>
        </div>

        <p class="text-gray-500 text-sm mt-6">
          By signing in, you agree to our Terms of Service
        </p>
      </div>

      <div class="text-center mt-6">
        <router-link to="/" class="text-gray-500 hover:text-white text-sm flex items-center justify-center gap-1">
          <ArrowLeftIcon class="w-4 h-4" />
          Back to home
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowLeftIcon } from '@heroicons/vue/24/outline'
import { getEnabledProviders, loginWithProvider, type IdentityProviderType } from '@/lib/identityProviders'
import GoogleIcon from '@/components/icons/GoogleIcon.vue'
import GitHubIcon from '@/components/icons/GitHubIcon.vue'
import MicrosoftIcon from '@/components/icons/MicrosoftIcon.vue'

const enabledProviders = getEnabledProviders()

function login(providerId: IdentityProviderType) {
  loginWithProvider(providerId)
}

function getProviderIcon(providerId: IdentityProviderType) {
  switch (providerId) {
    case 'google':
      return GoogleIcon
    case 'github':
      return GitHubIcon
    case 'microsoft':
      return MicrosoftIcon
    default:
      return 'span'
  }
}
</script>
