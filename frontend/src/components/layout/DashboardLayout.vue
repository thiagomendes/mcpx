<template>
  <div class="min-h-screen flex">
    <!-- Sidebar -->
    <aside class="w-64 border-r border-border p-4 flex flex-col">
      <!-- Logo -->
      <div class="flex items-center gap-3 mb-2 px-2">
        <img src="@/assets/mcpx-logo.svg" alt="mcpx" class="w-10 h-10 object-contain" />
        <span class="text-xl font-bold text-gradient">mcpx</span>
      </div>
      <!-- Beta Badge -->
      <div class="inline-flex w-fit items-center gap-1.5 px-2 py-1 mb-4 mx-2 rounded-md bg-purple-500/10 border border-purple-500/30">
        <BeakerIcon class="w-3.5 h-3.5 text-purple-400" />
        <span class="text-xs font-medium text-purple-300">Incubating</span>
      </div>

      <!-- Org Switcher -->
      <div class="mb-6 px-2">
        <OrgSwitcher />
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
        <!-- Logs Menu -->
        <div>
          <button 
            @click="logsMenuOpen = !logsMenuOpen"
            class="w-full flex items-center justify-between gap-3 px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-background-hover transition-colors"
            :class="{ '!text-white': isLogsActive }"
          >
            <div class="flex items-center gap-3">
              <ClipboardDocumentListIcon class="w-5 h-5" />
              Logs
            </div>
            <ChevronDownIcon class="w-4 h-4 transition-transform" :class="{ 'rotate-180': logsMenuOpen }" />
          </button>
          <div v-if="logsMenuOpen" class="ml-6 mt-1 space-y-1">
            <router-link 
              to="/logs/requests" 
              class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-background-hover transition-colors text-sm"
              active-class="!text-white !bg-background-hover"
            >
              Request Logs
            </router-link>
            <router-link 
              v-if="authStore.canAdmin"
              to="/logs/audit" 
              class="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-background-hover transition-colors text-sm"
              active-class="!text-white !bg-background-hover"
            >
              Audit Logs
            </router-link>
          </div>
        </div>
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
            <div class="flex items-center gap-2">
              <div class="text-sm font-medium truncate">{{ authStore.user?.name || 'User' }}</div>
              <!-- Provider Icon (Current Session) -->
              <component 
                v-if="authStore.authProvider"
                :is="getProviderIcon(authStore.authProvider)"
                class="w-3.5 h-3.5 text-gray-400"
                :title="authStore.authProvider"
              />
            </div>
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
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import OrgSwitcher from '@/components/OrgSwitcher.vue'
import { 
  Squares2X2Icon, 
  ServerIcon, 
  RectangleStackIcon,
  ClipboardDocumentListIcon,
  BellIcon,
  ArrowRightOnRectangleIcon,
  ChevronDownIcon,
  BeakerIcon
} from '@heroicons/vue/24/outline'
import GoogleIcon from '@/components/icons/GoogleIcon.vue'
import GitHubIcon from '@/components/icons/GitHubIcon.vue'
import MicrosoftIcon from '@/components/icons/MicrosoftIcon.vue'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

// Logs submenu state
const logsMenuOpen = ref(true)
const isLogsActive = computed(() => route.path.startsWith('/logs'))

function handleLogout() {
  authStore.logout()
  router.push('/login')
}

function getProviderIcon(provider: string) {
  switch (provider) {
    case 'google': return GoogleIcon
    case 'github': return GitHubIcon
    case 'microsoft': return MicrosoftIcon
    default: return null
  }
}
</script>
