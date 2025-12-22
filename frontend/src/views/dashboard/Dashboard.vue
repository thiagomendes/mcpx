<template>
  <DashboardLayout>
    <div class="mb-8">
      <h1 class="text-2xl font-bold mb-2">Dashboard</h1>
      <p class="text-gray-400">Welcome back, {{ authStore.user?.name || 'User' }}</p>
    </div>

    <!-- Servers Section -->
    <div class="mb-10">
      <div class="flex items-center gap-3 mb-4">
        <ServerIcon class="w-6 h-6 text-gray-400" />
        <h2 class="text-lg font-semibold">Servers</h2>
      </div>
      <div class="grid md:grid-cols-3 gap-6 mb-4">
        <div class="card">
          <div class="flex items-center gap-3 mb-2">
            <ServerIcon class="w-5 h-5 text-gray-400" />
            <span class="text-gray-400 text-sm">Total</span>
          </div>
          <div class="text-3xl font-bold">{{ serversStore.servers.length }}</div>
        </div>
        <div class="card">
          <div class="flex items-center gap-3 mb-2">
            <CheckCircleIcon class="w-5 h-5 text-green-400" />
            <span class="text-gray-400 text-sm">Active</span>
          </div>
          <div class="text-3xl font-bold text-green-400">
            {{ serversStore.servers.filter(s => s.enabled).length }}
          </div>
        </div>
        <div class="card">
          <div class="flex items-center gap-3 mb-2">
            <ArrowTrendingUpIcon class="w-5 h-5 text-gray-400" />
            <span class="text-gray-400 text-sm">Requests Today</span>
          </div>
          <div class="text-3xl font-bold">0</div>
        </div>
      </div>
      <div class="flex gap-4">
        <router-link to="/servers/new" class="btn btn-primary flex items-center gap-2">
          <PlusIcon class="w-5 h-5" />
          Add Server
        </router-link>
        <router-link to="/servers" class="btn btn-secondary flex items-center gap-2">
          <ServerIcon class="w-5 h-5" />
          View All
        </router-link>
      </div>
    </div>

    <!-- Gateways Section -->
    <div>
      <div class="flex items-center gap-3 mb-4">
        <RectangleStackIcon class="w-6 h-6 text-primary" />
        <h2 class="text-lg font-semibold">Virtual Gateways</h2>
      </div>
      <div class="grid md:grid-cols-3 gap-6 mb-4">
        <div class="card">
          <div class="flex items-center gap-3 mb-2">
            <RectangleStackIcon class="w-5 h-5 text-primary" />
            <span class="text-gray-400 text-sm">Total</span>
          </div>
          <div class="text-3xl font-bold text-primary">{{ gatewaysStore.gateways.length }}</div>
        </div>
        <div class="card">
          <div class="flex items-center gap-3 mb-2">
            <CheckCircleIcon class="w-5 h-5 text-green-400" />
            <span class="text-gray-400 text-sm">Active</span>
          </div>
          <div class="text-3xl font-bold text-green-400">
            {{ gatewaysStore.gateways.filter(g => g.enabled).length }}
          </div>
        </div>
        <div class="card">
          <div class="flex items-center gap-3 mb-2">
            <ArrowTrendingUpIcon class="w-5 h-5 text-gray-400" />
            <span class="text-gray-400 text-sm">Requests Today</span>
          </div>
          <div class="text-3xl font-bold">0</div>
        </div>
      </div>
      <div class="flex gap-4">
        <router-link to="/gateways?create=true" class="btn btn-primary flex items-center gap-2">
          <PlusIcon class="w-5 h-5" />
          Create Gateway
        </router-link>
        <router-link to="/gateways" class="btn btn-secondary flex items-center gap-2">
          <RectangleStackIcon class="w-5 h-5" />
          View All
        </router-link>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useAuthStore } from '@/stores/auth'
import { useServersStore } from '@/stores/servers'
import { useGatewaysStore } from '@/stores/gateways'
import { 
  ServerIcon, 
  CheckCircleIcon, 
  ArrowTrendingUpIcon, 
  PlusIcon,
  RectangleStackIcon
} from '@heroicons/vue/24/outline'

const authStore = useAuthStore()
const serversStore = useServersStore()
const gatewaysStore = useGatewaysStore()

onMounted(() => {
  serversStore.fetchServers()
  gatewaysStore.fetchGateways()
})
</script>
