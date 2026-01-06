<template>
  <div class="min-h-screen flex flex-col relative overflow-hidden">
    <!-- Animated Background -->
    <div class="absolute inset-0 overflow-hidden pointer-events-none">
      <div class="absolute -top-40 -right-40 w-96 h-96 bg-purple-500/20 rounded-full blur-3xl animate-pulse-slow"></div>
      <div class="absolute -bottom-40 -left-40 w-96 h-96 bg-cyan-500/20 rounded-full blur-3xl animate-pulse-slow animation-delay-2000"></div>
      <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-gradient-to-r from-purple-500/10 to-cyan-500/10 rounded-full blur-3xl"></div>
    </div>

    <!-- Header -->
    <header class="relative z-10 border-b border-white/10 backdrop-blur-sm">
      <div class="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <img src="@/assets/mcpx-logo.svg" alt="mcpx" class="w-10 h-10 object-contain" />
          <span class="text-xl font-bold text-white">mcpx</span>
        </div>
        <router-link to="/login" class="text-sm text-gray-400 hover:text-white transition-colors">
          Sign In →
        </router-link>
      </div>
    </header>

    <!-- Hero Section -->
    <main class="flex-1 relative z-10">
      <div class="max-w-7xl mx-auto px-6 py-24 text-center">
        <!-- Badge -->
        <div class="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white/5 border border-white/10 mb-8 animate-fade-in">
          <span class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></span>
          <span class="text-sm text-gray-300">Production Ready</span>
        </div>

        <!-- Main Headline -->
        <h1 class="text-5xl md:text-7xl font-bold mb-6 animate-fade-in-up">
          <span class="text-white">The </span>
          <span class="text-gradient">MCP Gateway</span>
          <br />
          <span class="text-white">for Enterprise</span>
        </h1>

        <p class="text-xl md:text-2xl text-gray-400 max-w-3xl mx-auto mb-12 animate-fade-in-up animation-delay-200">
          Bridge the gap between AI agents and your data. 
          <span class="text-white">Secure, scalable, ready for production.</span>
        </p>

        <!-- CTA Buttons -->
        <div class="flex flex-col sm:flex-row gap-4 justify-center mb-20 animate-fade-in-up animation-delay-400">
          <router-link to="/login" class="btn btn-primary px-8 py-4 text-lg font-semibold rounded-xl shadow-lg shadow-purple-500/25 hover:shadow-purple-500/40 transition-all hover:scale-105">
            Get Started Free
          </router-link>
          <a href="#features" class="px-8 py-4 text-lg font-semibold rounded-xl border border-white/20 text-white hover:bg-white/5 transition-all">
            Learn More
          </a>
        </div>

        <!-- Stats -->
        <div class="grid grid-cols-3 gap-8 max-w-2xl mx-auto mb-24 animate-fade-in-up animation-delay-600">
          <div class="text-center">
            <div class="text-3xl md:text-4xl font-bold text-white mb-1">100%</div>
            <div class="text-sm text-gray-500">Open Source</div>
          </div>
          <div class="text-center">
            <div class="text-3xl md:text-4xl font-bold text-gradient mb-1">&lt;5ms</div>
            <div class="text-sm text-gray-500">Proxy Latency</div>
          </div>
          <div class="text-center">
            <div class="text-3xl md:text-4xl font-bold text-white mb-1">∞</div>
            <div class="text-sm text-gray-500">MCP Servers</div>
          </div>
        </div>

        <!-- Features Grid -->
        <div id="features" class="grid md:grid-cols-3 gap-6">
          <div class="group card-glass p-8 rounded-2xl hover:scale-105 transition-all duration-300 hover:border-purple-500/50">
            <div class="w-14 h-14 mx-auto mb-6 rounded-xl bg-gradient-to-br from-purple-500/20 to-cyan-500/20 flex items-center justify-center group-hover:from-purple-500/30 group-hover:to-cyan-500/30 transition-all">
              <LinkIcon class="w-7 h-7 text-purple-400" />
            </div>
            <h3 class="text-xl font-semibold mb-3 text-white">Connect Instantly</h3>
            <p class="text-gray-400">Point to any MCP server via URL. Zero deployment, zero configuration.</p>
          </div>

          <div class="group card-glass p-8 rounded-2xl hover:scale-105 transition-all duration-300 hover:border-cyan-500/50">
            <div class="w-14 h-14 mx-auto mb-6 rounded-xl bg-gradient-to-br from-purple-500/20 to-cyan-500/20 flex items-center justify-center group-hover:from-purple-500/30 group-hover:to-cyan-500/30 transition-all">
              <ShieldCheckIcon class="w-7 h-7 text-cyan-400" />
            </div>
            <h3 class="text-xl font-semibold mb-3 text-white">Tool Governance</h3>
            <p class="text-gray-400">Allowlist or blocklist specific tools. Full control over what AI can access.</p>
          </div>

          <div class="group card-glass p-8 rounded-2xl hover:scale-105 transition-all duration-300 hover:border-purple-500/50">
            <div class="w-14 h-14 mx-auto mb-6 rounded-xl bg-gradient-to-br from-purple-500/20 to-cyan-500/20 flex items-center justify-center group-hover:from-purple-500/30 group-hover:to-cyan-500/30 transition-all">
              <ChartBarIcon class="w-7 h-7 text-purple-400" />
            </div>
            <h3 class="text-xl font-semibold mb-3 text-white">Full Observability</h3>
            <p class="text-gray-400">Audit logs, analytics, and real-time monitoring for every request.</p>
          </div>
        </div>

        <!-- More Features -->
        <div class="grid md:grid-cols-4 gap-4 mt-6">
          <div class="card-glass p-6 rounded-xl text-center hover:border-white/20 transition-all">
            <BoltIcon class="w-6 h-6 mx-auto mb-3 text-yellow-400" />
            <div class="text-sm font-medium text-white">Rate Limiting</div>
          </div>
          <div class="card-glass p-6 rounded-xl text-center hover:border-white/20 transition-all">
            <KeyIcon class="w-6 h-6 mx-auto mb-3 text-green-400" />
            <div class="text-sm font-medium text-white">OAuth & API Keys</div>
          </div>
          <div class="card-glass p-6 rounded-xl text-center hover:border-white/20 transition-all">
            <UsersIcon class="w-6 h-6 mx-auto mb-3 text-blue-400" />
            <div class="text-sm font-medium text-white">Multi-Org RBAC</div>
          </div>
          <div class="card-glass p-6 rounded-xl text-center hover:border-white/20 transition-all">
            <ServerStackIcon class="w-6 h-6 mx-auto mb-3 text-pink-400" />
            <div class="text-sm font-medium text-white">Gateway Aggregation</div>
          </div>
        </div>
      </div>
    </main>

    <!-- Footer -->
    <footer class="relative z-10 border-t border-white/10 py-8">
      <div class="max-w-7xl mx-auto px-6 flex flex-col md:flex-row items-center justify-between gap-4">
        <div class="flex items-center gap-3">
          <img src="@/assets/mcpx-logo.svg" alt="mcpx" class="w-8 h-8 object-contain opacity-60" />
          <span class="text-gray-500 text-sm">Built by TM Dev Lab</span>
        </div>
        <div class="flex gap-6 text-sm text-gray-500">
          <a href="#" class="hover:text-white transition-colors">Documentation</a>
          <a href="#" class="hover:text-white transition-colors">GitHub</a>
        </div>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { 
  LinkIcon, 
  ShieldCheckIcon, 
  ChartBarIcon,
  BoltIcon,
  KeyIcon,
  UsersIcon,
  ServerStackIcon
} from '@heroicons/vue/24/outline'

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

<style scoped>
.card-glass {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(10px);
}

.animate-pulse-slow {
  animation: pulse 4s ease-in-out infinite;
}

.animation-delay-2000 {
  animation-delay: 2s;
}

.animation-delay-200 {
  animation-delay: 0.2s;
}

.animation-delay-400 {
  animation-delay: 0.4s;
}

.animation-delay-600 {
  animation-delay: 0.6s;
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes fade-in-up {
  from { 
    opacity: 0; 
    transform: translateY(20px);
  }
  to { 
    opacity: 1; 
    transform: translateY(0);
  }
}

.animate-fade-in {
  animation: fade-in 0.6s ease-out forwards;
}

.animate-fade-in-up {
  animation: fade-in-up 0.6s ease-out forwards;
  opacity: 0;
}
</style>
