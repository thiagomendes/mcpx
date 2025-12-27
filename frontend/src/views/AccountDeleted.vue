<template>
  <div class="min-h-screen flex items-center justify-center bg-background p-4">
    <div class="max-w-md w-full text-center">
      <!-- Icon -->
      <div class="mx-auto w-20 h-20 rounded-full bg-violet-500/20 flex items-center justify-center mb-6">
        <CheckCircleIcon class="w-10 h-10 text-violet-400" />
      </div>

      <!-- Title -->
      <h1 class="text-2xl font-bold mb-2">Account Deleted</h1>
      <p class="text-gray-400 mb-8">
        Your account and all associated data have been permanently deleted. 
        We're sorry to see you go.
      </p>

      <!-- Call to action -->
      <a 
        href="/" 
        class="btn btn-primary inline-flex items-center gap-2"
      >
        <HomeIcon class="w-5 h-5" />
        Back to Homepage
      </a>

      <!-- Footer note -->
      <p class="text-xs text-gray-500 mt-8">
        If you change your mind, you can always create a new account.
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CheckCircleIcon, HomeIcon } from '@heroicons/vue/24/outline'
</script>
