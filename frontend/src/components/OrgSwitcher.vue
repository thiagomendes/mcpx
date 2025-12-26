<template>
  <div class="relative" ref="dropdownRef">
    <button 
      @click="isOpen = !isOpen"
      class="flex items-center gap-2 px-3 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 transition-colors text-sm"
    >
      <BuildingOfficeIcon v-if="!currentOrg?.is_personal" class="w-4 h-4 text-gray-400" />
      <UserIcon v-else class="w-4 h-4 text-gray-400" />
      <span class="font-medium">{{ currentOrg?.name || 'Select org' }}</span>
      <ChevronDownIcon class="w-4 h-4 text-gray-400" />
    </button>

    <!-- Dropdown -->
    <Transition
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <div 
        v-if="isOpen"
        class="absolute left-0 mt-2 w-full min-w-[240px] bg-gray-800 border border-gray-700 rounded-lg shadow-xl z-50"
      >
        <div class="p-2 border-b border-gray-700">
          <p class="text-xs text-gray-400 uppercase tracking-wide px-2 py-1">Switch Organization</p>
        </div>
        
        <div class="max-h-64 overflow-y-auto p-2">
          <button
            v-for="org in orgs"
            :key="org.id"
            @click="switchTo(org)"
            class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left transition-colors"
            :class="org.id === currentOrg?.id ? 'bg-violet-600/20 text-violet-400' : 'hover:bg-gray-700'"
          >
            <BuildingOfficeIcon v-if="!org.is_personal" class="w-5 h-5 text-gray-400 flex-shrink-0" />
            <UserIcon v-else class="w-5 h-5 text-gray-400 flex-shrink-0" />
            <div class="min-w-0 flex-1">
              <p class="font-medium truncate">{{ org.name }}</p>
              <p class="text-xs text-gray-500">{{ org.is_personal ? 'Personal' : org.role }}</p>
            </div>
            <CheckIcon v-if="org.id === currentOrg?.id" class="w-4 h-4 text-violet-400 flex-shrink-0" />
          </button>
        </div>

        <div class="p-2 border-t border-gray-700 space-y-1">
          <button 
            @click="showCreateModal = true; isOpen = false"
            class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
          >
            <PlusIcon class="w-4 h-4" />
            Create New Organization
          </button>
          
          <router-link 
            to="/settings"
            @click="isOpen = false"
            class="flex items-center gap-2 px-3 py-2 text-sm text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
          >
            <Cog6ToothIcon class="w-4 h-4" />
            Organization Settings
          </router-link>
        </div>
      </div>
    </Transition>

    <CreateOrgModal 
      v-if="showCreateModal" 
      @close="showCreateModal = false"
      @created="handleOrgCreated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useAuthStore } from '@/stores/auth'
import CreateOrgModal from '@/components/CreateOrgModal.vue'
import { 
  BuildingOfficeIcon, 
  UserIcon, 
  ChevronDownIcon, 
  CheckIcon,
  Cog6ToothIcon,
  PlusIcon
} from '@heroicons/vue/24/outline'

// import { useRouter } from 'vue-router'

// const router = useRouter()
const authStore = useAuthStore()
const isOpen = ref(false)
const showCreateModal = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

const currentOrg = computed(() => authStore.currentOrg)
const orgs = computed(() => authStore.orgs)

async function switchTo(org: { id: string }) {
  if (org.id !== currentOrg.value?.id) {
    await authStore.switchOrg(org.id)
    window.location.reload()
  }
  isOpen.value = false
}

function handleOrgCreated() {
  // Navigate to settings to show fresh state of new org
  window.location.href = '/settings'
}

function handleClickOutside(event: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>
