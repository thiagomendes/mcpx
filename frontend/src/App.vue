<template>
  <div class="min-h-screen bg-background-darker">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { watch, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useRouter, useRoute } from 'vue-router'

const authStore = useAuthStore()
const router = useRouter()
const route = useRoute()

watch(
  () => route.query.token,
  async (token) => {
    if (token && typeof token === 'string') {
      console.log('Token found in URL, saving and redirecting...')
      authStore.setToken(token)
      await router.replace('/dashboard')
    }
  },
  { immediate: true }
)

onMounted(async () => {
  // Try to load user if we have a token
  if (authStore.token && !authStore.user) {
    try {
      await authStore.fetchUser()
    } catch (e) {
      console.error('Failed to fetch user:', e)
      authStore.logout()
    }
  }
})
</script>
