<template>
  <div class="min-h-screen flex">
    <!-- Sidebar -->
    <aside class="w-64 border-r border-border p-4 flex flex-col">
      <!-- Logo -->
      <div class="flex items-center gap-3 mb-8 px-2">
        <img src="@/assets/mcpx-logo.svg" alt="mcpx" class="w-10 h-10 object-contain" />
        <span class="text-xl font-bold text-gradient">mcpx</span>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 space-y-1">
        <router-link 
          to="/dashboard" 
          class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-background-hover transition-colors"
          active-class="!text-white !bg-background-hover"
        >
          <Squares2X2Icon class="w-5 h-5" />
          Dashboard
        </router-link>
        <router-link 
          to="/servers" 
          class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-background-hover transition-colors"
          active-class="!text-white !bg-background-hover"
        >
          <ServerIcon class="w-5 h-5" />
          Servers
        </router-link>
        <router-link 
          to="/gateways" 
          class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-background-hover transition-colors"
          active-class="!text-white !bg-background-hover"
        >
          <RectangleStackIcon class="w-5 h-5" />
          Gateways
        </router-link>
        <router-link 
          to="/audit" 
          class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-background-hover transition-colors"
          active-class="!text-white !bg-background-hover"
        >
          <ClipboardDocumentListIcon class="w-5 h-5" />
          Audit Logs
        </router-link>
        <router-link 
          to="/alerts" 
          class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-background-hover transition-colors"
          active-class="!text-white !bg-background-hover"
        >
          <BellIcon class="w-5 h-5" />
          Alerts
        </router-link>
      </nav>

      <!-- User -->
      <div class="border-t border-border pt-4 mt-4">
        <div class="flex items-center gap-3 px-2">
          <img 
            v-if="authStore.user?.avatar_url" 
            :src="authStore.user.avatar_url" 
            class="w-8 h-8 rounded-full"
          />
          <div v-else class="w-8 h-8 rounded-full bg-primary flex items-center justify-center text-white text-sm font-medium">
            {{ authStore.user?.email?.[0]?.toUpperCase() || 'U' }}
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium truncate">{{ authStore.user?.name || 'User' }}</div>
            <div class="text-xs text-gray-500 truncate">{{ authStore.user?.email }}</div>
          </div>
        </div>
        <button 
          @click="handleLogout"
          class="w-full mt-3 px-3 py-2 text-sm text-gray-400 hover:text-white hover:bg-background-hover rounded-lg transition-colors text-left flex items-center gap-2"
        >
          <ArrowRightOnRectangleIcon class="w-4 h-4" />
          Sign out
        </button>
      </div>
    </aside>

    <!-- Main content -->
    <main class="flex-1 p-8 overflow-auto">
      <slot />
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { 
  Squares2X2Icon, 
  ServerIcon, 
  RectangleStackIcon,
  ClipboardDocumentListIcon,
  BellIcon,
  ArrowRightOnRectangleIcon 
} from '@heroicons/vue/24/outline'

const router = useRouter()
const authStore = useAuthStore()

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>
