<template>
  <DashboardLayout>
    <div class="flex items-center justify-between mb-8">
      <div>
        <h1 class="text-2xl font-bold mb-2">Virtual Gateways</h1>
        <p class="text-gray-400">Aggregate multiple servers into single endpoints</p>
      </div>
      <button @click="showCreateModal = true" class="btn btn-primary flex items-center gap-2">
        <PlusIcon class="w-5 h-5" />
        New Gateway
      </button>
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
      <button @click="showCreateModal = true" class="btn btn-primary inline-flex items-center gap-2">
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
import { 
  RectangleStackIcon, 
  PlusIcon, 
  ClipboardDocumentIcon, 
  TrashIcon 
} from '@heroicons/vue/24/outline'

const route = useRoute()
const gatewaysStore = useGatewaysStore()
const showCreateModal = ref(false)
const creating = ref(false)
const newGateway = reactive({ name: '', slug: '' })

onMounted(() => {
  gatewaysStore.fetchGateways()
  
  // Open create modal if ?create=true query param is present
  if (route.query.create === 'true') {
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
