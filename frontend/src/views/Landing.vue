<template>
  <div class="min-h-screen flex flex-col">
    <!-- Header (minimal - just border) -->
    <header class="border-b border-border">
      <div class="max-w-7xl mx-auto px-4 py-4">
      </div>
    </header>

    <!-- Hero -->
    <main class="flex-1 flex items-center">
      <div class="max-w-7xl mx-auto px-4 py-20 text-center">
        <!-- Logo -->
        <div class="flex flex-col items-center mb-8">
          <img src="@/assets/mcpx-logo.svg" alt="mcpx" class="w-24 h-24 object-contain" />
          <span class="text-5xl md:text-6xl font-bold text-white mt-3">mcpx</span>
        </div>
        <h1 class="text-5xl md:text-6xl font-bold mb-6">
          <span class="text-gradient">The Production-Ready</span>
          <br />
          <span class="text-white">MCP Gateway</span>
        </h1>
        <p class="text-xl text-gray-400 max-w-2xl mx-auto mb-10">
          Bridge the gap between AI agents and your data. 
          Secure, scalable, ready for production.
        </p>
        <div class="flex gap-4 justify-center">
          <router-link to="/login" class="btn btn-primary px-8 py-3 text-lg">
            Start Free
          </router-link>
        </div>

        <!-- Features Grid -->
        <div class="grid md:grid-cols-3 gap-6 mt-20">
          <div class="card">
            <div class="w-12 h-12 mx-auto mb-4 rounded-lg bg-primary/20 flex items-center justify-center">
              <LinkIcon class="w-6 h-6 text-primary" />
            </div>
            <h3 class="text-lg font-semibold mb-2">Connect Instantly</h3>
            <p class="text-gray-400 text-sm">Point to any MCP server via URL. No deployment required.</p>
          </div>
          <div class="card">
            <div class="w-12 h-12 mx-auto mb-4 rounded-lg bg-primary/20 flex items-center justify-center">
              <ShieldCheckIcon class="w-6 h-6 text-primary" />
            </div>
            <h3 class="text-lg font-semibold mb-2">Tool Governance</h3>
            <p class="text-gray-400 text-sm">Whitelist/blacklist specific tools. Control what AI can access.</p>
          </div>
          <div class="card">
            <div class="w-12 h-12 mx-auto mb-4 rounded-lg bg-primary/20 flex items-center justify-center">
              <ChartBarIcon class="w-6 h-6 text-primary" />
            </div>
            <h3 class="text-lg font-semibold mb-2">Complete Observability</h3>
            <p class="text-gray-400 text-sm">Audit logs, analytics, and real-time monitoring.</p>
          </div>
        </div>
      </div>
    </main>

    <!-- Footer -->
    <footer class="border-t border-border py-6">
      <div class="max-w-7xl mx-auto px-4 text-center text-gray-500 text-sm">
        Built by TM Dev Lab
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { LinkIcon, ShieldCheckIcon, ChartBarIcon } from '@heroicons/vue/24/outline'

const router = useRouter()
const authStore = useAuthStore()

onMounted(() => {
  // Check for token in URL (from OAuth redirect)
  const urlParams = new URLSearchParams(window.location.search)
  const token = urlParams.get('token')
  
  if (token) {
    authStore.setToken(token)
    // Clean URL
    window.history.replaceState({}, document.title, window.location.pathname)
    
    // Check for pending auth redirect (e.g. from invite link)
    const redirect = localStorage.getItem('auth_redirect')
    if (redirect) {
      localStorage.removeItem('auth_redirect')
      router.push(redirect)
    } else {
      router.push('/dashboard')
    }
  }
})
</script>
