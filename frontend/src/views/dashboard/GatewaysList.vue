<template>
  <DashboardLayout>
    <div class="flex items-center justify-between mb-8">
      <div>
        <h1 class="text-2xl font-bold mb-2">Virtual Gateways</h1>
        <p class="text-gray-400">Aggregate multiple servers into single endpoints</p>
      </div>
      <button 
        v-if="canWrite"
        @click="showCreateModal = true" 
        :disabled="limitReached"
        :class="['btn flex items-center gap-2', limitReached ? 'btn-secondary opacity-50 cursor-not-allowed' : 'btn-primary']"
      >
        <PlusIcon class="w-5 h-5" />
        New Gateway
      </button>
    </div>

    <!-- Limit Warning (only for admins) -->
    <div v-if="limitReached && canWrite" class="card bg-amber-500/20 border-amber-500/50 mb-4">
      <div class="flex items-start gap-3">
        <NoSymbolIcon class="w-5 h-5 text-amber-400 shrink-0" />
        <p class="text-gray-300 text-sm">
          You have {{ limitInfo.current }}/{{ limitInfo.max }} gateways. Delete one or increase the limit in 
          <router-link to="/settings/configuration" class="text-violet-400 hover:underline">Configuration</router-link>.
        </p>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="gatewaysStore.loading" class="text-center py-12">
      <div class="text-gray-400">Loading gateways...</div>
    </div>

    <!-- Empty State -->
    <div v-else-if="gatewaysStore.gateways.length === 0" class="card text-center py-12">
      <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-primary/20 flex items-center justify-center">
        <RectangleStackIcon class="w-8 h-8 text-primary" />
      </div>
      <h3 class="text-lg font-semibold mb-2">No gateways yet</h3>
      <p class="text-gray-400 mb-6">Create a virtual gateway to aggregate multiple MCP servers</p>
      <button 
        @click="showCreateModal = true" 
        :disabled="limitReached"
        :class="['btn inline-flex items-center gap-2', limitReached ? 'btn-secondary opacity-50 cursor-not-allowed' : 'btn-primary']"
      >
        <PlusIcon class="w-5 h-5" />
        Create Your First Gateway
      </button>
    </div>

    <!-- Gateway List -->
    <div v-else class="space-y-4">
      <div 
        v-for="gateway in gatewaysStore.gateways" 
        :key="gateway.id"
        class="card flex items-center justify-between hover:border-primary cursor-pointer"
        @click="$router.push(`/gateways/${gateway.slug}`)"
      >
        <div class="flex-1">
          <div class="flex items-center gap-3 mb-1">
            <span class="font-semibold">{{ gateway.name }}</span>
            <span :class="gateway.enabled ? 'badge badge-success' : 'badge badge-error'">
              {{ gateway.enabled ? 'Enabled' : 'Disabled' }}
            </span>
            <span class="badge badge-info">{{ gateway.servers.length }} servers</span>
          </div>
          <div class="text-gray-400 text-sm font-mono">{{ gateway.proxy_url }}</div>
          <div v-if="gateway.servers.length > 0" class="flex gap-2 mt-2">
            <span 
              v-for="server in gateway.servers" 
              :key="server.name"
              class="text-xs px-2 py-1 bg-gray-700 rounded"
            >
              {{ server.name }}
            </span>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <button 
            @click.stop="copyProxyUrl(gateway.proxy_url)" 
            class="btn btn-ghost text-sm flex items-center gap-1"
            title="Copy proxy URL"
          >
            <ClipboardDocumentIcon class="w-4 h-4" />
            Copy URL
          </button>
          <button 
            v-if="canWrite"
            @click.stop="handleDelete(gateway.slug, gateway.name)" 
            class="btn btn-ghost text-sm text-red-400 hover:text-red-300 flex items-center gap-1"
          >
            <TrashIcon class="w-4 h-4" />
            Delete
          </button>
        </div>
      </div>
    </div>

    <!-- Create Modal -->
    <div v-if="showCreateModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div class="card w-full max-w-md">
        <h3 class="text-lg font-semibold mb-4">Create Gateway</h3>
        <form @submit.prevent="handleCreate">
          <div class="mb-4">
            <label class="block text-sm font-medium mb-2">Name</label>
            <input 
              v-model="newGateway.name" 
              type="text" 
              class="input w-full" 
              placeholder="My Gateway"
              required
            />
          </div>
          <div class="mb-6">
            <label class="block text-sm font-medium mb-2">Slug</label>
            <input 
              v-model="newGateway.slug" 
              type="text" 
              class="input w-full" 
              placeholder="my-gateway"
              pattern="[a-zA-Z0-9_-]+"
              required
            />
            <p class="text-xs text-gray-400 mt-1">Used in the proxy URL (alphanumeric, hyphens, underscores)</p>
          </div>
          <div class="flex gap-3 justify-end">
            <button type="button" @click="showCreateModal = false" class="btn btn-ghost">Cancel</button>
            <button type="submit" class="btn btn-primary" :disabled="creating">
              {{ creating ? 'Creating...' : 'Create Gateway' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { onMounted, ref, reactive } from 'vue'
import { useRoute } from 'vue-router'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import { useGatewaysStore } from '@/stores/gateways'

import { usePermissions } from '@/composables/usePermissions'
import { 
  RectangleStackIcon, 
  PlusIcon, 
  ClipboardDocumentIcon, 
  TrashIcon,
  NoSymbolIcon
} from '@heroicons/vue/24/outline'

const route = useRoute()
const gatewaysStore = useGatewaysStore()

const { canWrite } = usePermissions()
const showCreateModal = ref(false)
const creating = ref(false)
const newGateway = reactive({ name: '', slug: '' })
const limitReached = ref(false)
const limitInfo = ref({ current: 0, max: 0 })

async function checkLimits() {
  try {
    const api = (await import('@/api/client')).default
    const response = await api.get('/limits')
    const data = response.data
    limitInfo.value = { current: data.gateways.current, max: data.gateways.max }
    limitReached.value = !data.gateways.can_create
  } catch (e) {
    console.error('Failed to check limits:', e)
  }
}

onMounted(() => {
  gatewaysStore.fetchGateways()
  checkLimits()
  
  // Open create modal if ?create=true query param is present
  if (route.query.create === 'true' && !limitReached.value) {
    showCreateModal.value = true
  }
})

function copyProxyUrl(url: string) {
  navigator.clipboard.writeText(url)
  alert('Proxy URL copied to clipboard!')
}

async function handleDelete(slug: string, name: string) {
  if (confirm(`Delete gateway "${name}"? This cannot be undone.`)) {
    await gatewaysStore.deleteGateway(slug)
  }
}

async function handleCreate() {
  creating.value = true
  try {
    await gatewaysStore.createGateway({
      name: newGateway.name,
      slug: newGateway.slug,
    })
    showCreateModal.value = false
    newGateway.name = ''
    newGateway.slug = ''
  } catch {
    alert('Failed to create gateway')
  } finally {
    creating.value = false
  }
}
</script>
