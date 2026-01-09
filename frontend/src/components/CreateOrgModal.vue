<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="$emit('close')">
    <div class="card w-full max-w-md">
      <h2 class="text-lg font-semibold mb-4">Create New Organization</h2>
      
      <form @submit.prevent="handleSubmit">
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-400 mb-1">Organization Name</label>
          <input 
            v-model="name"
            type="text" 
            class="input w-full"
            placeholder="e.g. Acme Corp"
            required
            autofocus
          />
          <p class="text-xs text-gray-500 mt-1">
            We'll generate a URL-safe slug for you automatically.
          </p>
        </div>

        <div v-if="error" class="mb-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-sm">
          {{ error }}
        </div>

        <div class="flex items-center gap-3">
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
              class="btn btn-primary flex-1 whitespace-nowrap"
              :disabled="loading || !name.trim()"
            >
              <span v-if="loading">Creating...</span>
              <span v-else>Create Organization</span>
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '@/stores/auth'
import api from '@/api/client'

const emit = defineEmits(['close', 'created'])
const authStore = useAuthStore()

const name = ref('')
const loading = ref(false)
const error = ref<string | null>(null)

async function handleSubmit() {
  if (!name.value.trim()) return

  loading.value = true
  error.value = null

  try {
    const response = await api.post('/orgs', {
      name: name.value.trim()
    })
    
    const newOrg = response.data
    
    // Switch to new org immediately
    await authStore.fetchUser() // Refresh org list
    await authStore.switchOrg(newOrg.id)
    
    emit('created', newOrg)
    emit('close')
  } catch (err: unknown) {
    const error_obj = err as { response?: { data?: { message?: string } } }
    console.error('Failed to create org:', err)
    error.value = error_obj.response?.data?.message || 'Failed to create organization'
  } finally {
    loading.value = false
  }
}
</script>
