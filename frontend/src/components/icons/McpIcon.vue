<template>
  <svg 
    :width="size" 
    :height="size" 
    viewBox="0 0 180 180" 
    fill="none" 
    xmlns="http://www.w3.org/2000/svg"
  >
    <defs>
      <linearGradient id="mcpGradient" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="#8B5CF6" />
        <stop offset="100%" stop-color="#06B6D4" />
      </linearGradient>
    </defs>
    <!-- MCP Protocol paths -->
    <path 
      d="M23.5996 85.2532L86.2021 22.6507C94.8457 14.0071 108.86 14.0071 117.503 22.6507C126.147 31.2942 126.147 45.3083 117.503 53.9519L70.2254 101.23" 
      :stroke="useGradient ? 'url(#mcpGradient)' : color" 
      stroke-width="11.0667" 
      stroke-linecap="round"
    />
    <path 
      d="M70.8789 100.578L117.504 53.952C126.148 45.3083 140.163 45.3083 148.806 53.952L149.132 54.278C157.776 62.9216 157.776 76.9357 149.132 85.5792L92.5139 142.198C89.6327 145.079 89.6327 149.75 92.5139 152.631L104.14 164.257" 
      :stroke="useGradient ? 'url(#mcpGradient)' : color" 
      stroke-width="11.0667" 
      stroke-linecap="round"
    />
    <path 
      d="M101.853 38.3013L55.553 84.6011C46.9094 93.2447 46.9094 107.258 55.553 115.902C64.1966 124.546 78.2106 124.546 86.8543 115.902L133.154 69.6025" 
      :stroke="useGradient ? 'url(#mcpGradient)' : color" 
      stroke-width="11.0667" 
      stroke-linecap="round"
    />
    <!-- X overlay - fused with lower right of MCP design -->
    <path 
      d="M120 130L160 170" 
      :stroke="useGradient ? 'url(#mcpGradient)' : color" 
      stroke-width="11.0667" 
      stroke-linecap="round"
    />
    <path 
      d="M160 130L120 170" 
      :stroke="useGradient ? 'url(#mcpGradient)' : color" 
      stroke-width="11.0667" 
      stroke-linecap="round"
    />
  </svg>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  size?: number
  color?: string
  useGradient?: boolean
}>(), {
  size: 24,
  color: 'white',
  useGradient: false
})
</script>
