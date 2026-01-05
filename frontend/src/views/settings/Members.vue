<template>
  <DashboardLayout>
    <div>
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-2xl font-bold mb-1">Team Members</h1>
          <p class="text-gray-400">Manage who has access to {{ currentOrg?.name }}</p>
        </div>
        <div class="flex items-center gap-3">
          <button 
            v-if="!currentOrg?.is_personal && canAdmin"
            @click="showInviteModal = true"
            class="btn btn-primary flex items-center gap-2"
          >
            <PlusIcon class="w-4 h-4" />
            Invite Member
          </button>
          
          <div 
            v-if="currentOrg?.is_personal" 
            class="px-3 py-1.5 bg-gray-800 rounded-lg border border-gray-700 text-xs text-gray-400 flex items-center gap-2"
          >
            <UserIcon class="w-3 h-3" />
            Personal Organization (Single User)
          </div>
        </div>
      </div>

      <!-- Members List -->
      <div class="card">
        <div class="divide-y divide-gray-700">
          <div 
            v-for="member in members" 
            :key="member.user_id"
            class="flex items-center justify-between py-4 first:pt-0 last:pb-0"
          >
            <div class="flex items-center gap-4">
              <img 
                v-if="member.avatar_url" 
                :src="member.avatar_url" 
                class="w-10 h-10 rounded-full"
              />
              <div v-else class="w-10 h-10 rounded-full bg-violet-500/20 flex items-center justify-center text-violet-400 font-medium">
                {{ member.email[0].toUpperCase() }}
              </div>
              <div>
                <div class="font-medium">{{ member.name || member.email }}</div>
                <div class="text-sm text-gray-400">{{ member.email }}</div>
              </div>
            </div>
            <div class="flex items-center gap-4">
              <span 
                class="px-2 py-1 text-xs rounded-full capitalize"
                :class="{
                  'bg-amber-500/20 text-amber-400': member.role === 'owner',
                  'bg-violet-500/20 text-violet-400': member.role === 'admin',
                  'bg-gray-500/20 text-gray-400': member.role === 'member',
                }"
              >
                {{ member.role }}
              </span>
              
              <button 
                v-if="canAdmin && member.user_id !== authStore.user?.id"
                @click="removeMember(member)"
                class="p-2 text-gray-500 hover:text-red-400 transition-colors"
                title="Remove member"
              >
                <TrashIcon class="w-5 h-5" />
              </button>
            </div>
          </div>
        </div>

        <div v-if="members.length === 0" class="text-center py-8 text-gray-400">
          No members yet
        </div>
      </div>

      <!-- Back link -->
      <div class="mt-6">
        <router-link 
          to="/settings"
          class="text-gray-400 hover:text-white text-sm flex items-center gap-1"
        >
          <ArrowLeftIcon class="w-4 h-4" />
          Back to Settings
        </router-link>
      </div>
    </div>

    <!-- Invite Modal -->
    <InviteMemberModal 
      v-if="showInviteModal" 
      @close="showInviteModal = false"
      @invited="loadMembers"
    />
  </DashboardLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import InviteMemberModal from '@/components/InviteMemberModal.vue'
import { PlusIcon, ArrowLeftIcon, UserIcon, TrashIcon } from '@heroicons/vue/24/outline'
import api from '@/api/client'

interface Member {
  user_id: string
  email: string
  name: string | null
  avatar_url: string | null
  role: string
  joined_at: string
}

const authStore = useAuthStore()
const showInviteModal = ref(false)
const members = ref<Member[]>([])

const currentOrg = computed(() => authStore.currentOrg)
const canAdmin = computed(() => authStore.canAdmin)

async function loadMembers() {
  try {
    const response = await api.get<Member[]>('/orgs/members')
    members.value = response.data
  } catch (error) {
    console.error('Failed to load members:', error)
  }
}

async function removeMember(member: Member) {
  if (!confirm(`Are you sure you want to remove ${member.name || member.email} from the organization?`)) {
    return
  }
  
  try {
    await api.delete(`/orgs/members/${member.user_id}`)
    await loadMembers()
  } catch (error: unknown) {
    const error_obj = error as { response?: { data?: { message?: string } } }
    console.error('Failed to remove member:', error)
    alert(error_obj.response?.data?.message || 'Failed to remove member')
  }
}

onMounted(() => {
  loadMembers()
})
</script>
