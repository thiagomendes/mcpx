<template>
  <div class="min-h-screen flex items-center justify-center">
    <div class="max-w-md w-full px-4">
      <div class="card text-center">
        <!-- Logo -->
        <div class="flex items-center justify-center gap-3 mb-8">
          <div class="w-12 h-12 bg-gradient-primary rounded-lg flex items-center justify-center">
            <BoltIcon class="w-7 h-7 text-white" />
          </div>
          <span class="text-2xl font-bold text-gradient">mcpx</span>
        </div>

        <h1 class="text-2xl font-bold mb-2">Welcome back</h1>
        <p class="text-gray-400 mb-8">Sign in to manage your MCP servers</p>

        <!-- Google Sign In Button -->
        <a 
          href="/api/auth/google" 
          class="w-full flex items-center justify-center gap-3 px-4 py-3 bg-white text-gray-900 rounded-lg font-medium hover:bg-gray-100 transition-colors"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24">
            <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
            <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
            <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
            <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
          </svg>
          Sign in with Google
        </a>

        <p class="text-gray-500 text-sm mt-6">
          By signing in, you agree to our Terms of Service
        </p>
      </div>

      <div class="text-center mt-6">
        <router-link to="/" class="text-gray-500 hover:text-white text-sm flex items-center justify-center gap-1">
          <ArrowLeftIcon class="w-4 h-4" />
          Back to home
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BoltIcon, ArrowLeftIcon } from '@heroicons/vue/24/outline'
</script>
