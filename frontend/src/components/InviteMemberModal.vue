<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="$emit('close')">
    <div class="card w-full max-w-md">
      <h2 class="text-lg font-semibold mb-4">Invite Team Member</h2>
      
      <form @submit.prevent="handleSubmit" v-if="!result">
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-400 mb-1">Email Address</label>
          <input 
            v-model="email"
            type="email" 
            class="input w-full"
            placeholder="colleague@example.com"
            required
            autofocus
            :disabled="loading"
          />
        </div>

        <div class="mb-6">
          <label class="block text-sm font-medium text-gray-400 mb-1">Role</label>
          <select v-model="role" class="input w-full" :disabled="loading">
            <option value="member">Member</option>
            <option value="admin">Admin</option>
          </select>
          <p class="text-xs text-gray-500 mt-1">
            Admins can manage servers, gateways, and other members.
          </p>
        </div>

        <div v-if="error" class="mb-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-sm">
          {{ error }}
        </div>

        <div class="flex items-center gap-3">
          <button 
            type="button" 
            @click="$emit('close')"
            class="btn btn-secondary flex-1"
            :disabled="loading"
          >
            Cancel
          </button>
          <button 
            type="submit" 
            class="btn btn-primary flex-1"
            :disabled="loading || !email"
          >
            <span v-if="loading">Inviting...</span>
            <span v-else>Send Invite</span>
          </button>
        </div>
      </form>

      <!-- Success State -->
      <div v-else class="text-center">
        <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-green-500/20 flex items-center justify-center">
          <CheckIcon class="w-8 h-8 text-green-400" />
        </div>
        <h3 class="text-lg font-semibold mb-2">Invite Sent!</h3>
        <p class="text-gray-400 mb-6">{{ result.message }}</p>

        <div v-if="result.invite_link" class="mb-6 text-left">
          <label class="block text-xs font-medium text-gray-500 mb-1 uppercase">Invite Link</label>
          <div class="flex items-center gap-2">
            <input 
              readonly 
              :value="result.invite_link"
              class="input w-full text-sm font-mono bg-gray-800"
              @click="($event.target as HTMLInputElement).select()"
            />
            <button 
              @click="copyLink"
              class="btn btn-secondary p-2"
              title="Copy Link"
            >
              <ClipboardDocumentIcon v-if="!copied" class="w-5 h-5" />
              <CheckIcon v-else class="w-5 h-5 text-green-400" />
            </button>
          </div>
          <p class="text-xs text-gray-500 mt-2">
            Share this link with them to join the organization.
          </p>
        </div>

        <button 
          @click="$emit('invited'); $emit('close')"
          class="btn btn-primary w-full"
        >
          Done
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import api from '@/api/client'
import { CheckIcon, ClipboardDocumentIcon } from '@heroicons/vue/24/outline'

const _emit = defineEmits(['close', 'invited'])

const email = ref('')
const role = ref('member')
const loading = ref(false)
const error = ref<string | null>(null)
const result = ref<{ message: string; invite_link?: string } | null>(null)
const copied = ref(false)

async function handleSubmit() {
  if (!email.value) return

  loading.value = true
  error.value = null

  try {
    const response = await api.post('/orgs/members/invite', {
      email: email.value,
      role: role.value
    })
    result.value = response.data
  } catch (err: unknown) {
    const error_obj = err as { response?: { data?: { message?: string } } }
    console.error('Failed to invite member:', err)
    error.value = error_obj.response?.data?.message || 'Failed to send invite'
  } finally {
    loading.value = false
  }
}

function copyLink() {
  if (result.value?.invite_link) {
    navigator.clipboard.writeText(result.value.invite_link)
    copied.value = true
    setTimeout(() => copied.value = false, 2000)
  }
}
</script>
