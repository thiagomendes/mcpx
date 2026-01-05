<template>
  <div class="card mb-6">
    <div class="flex items-center justify-between mb-4">
      <h3 class="font-semibold flex items-center gap-2">
        <FunnelIcon class="w-5 h-5 text-primary" />
        Tool Governance
      </h3>
      <button 
        v-if="hasConfig" 
        @click="handleClear" 
        class="text-sm text-gray-400 hover:text-red-400 transition-colors"
      >
        Clear Rules
      </button>
    </div>

    <!-- Prefix Section -->
    <div class="mb-6 pb-4 border-b border-gray-700">
      <label class="text-sm text-gray-400 mb-2 block">Tool Prefix (applies to all tools)</label>
      <div class="flex gap-2 items-center">
        <input 
          v-model="prefix" 
          type="text" 
          placeholder="e.g. github"
          class="flex-1 bg-background-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
        />
        <span v-if="prefix" class="text-gray-500 text-sm">→ {{ prefix }}_tool_name</span>
      </div>
    </div>
    
    <!-- Limit Toggle -->
    <div class="mb-4">
      <label class="flex items-center gap-3 cursor-pointer">
        <div 
          @click="limitEnabled = !limitEnabled"
          :class="[
            'w-11 h-6 rounded-full transition-colors relative',
            limitEnabled ? 'bg-primary' : 'bg-gray-600'
          ]"
        >
          <div 
            :class="[
              'absolute top-1 w-4 h-4 bg-white rounded-full transition-transform',
              limitEnabled ? 'translate-x-6' : 'translate-x-1'
            ]"
          ></div>
        </div>
        <span class="text-sm">Limit tool exposure</span>
      </label>
      <p v-if="!limitEnabled" class="text-gray-500 text-xs mt-2">All tools from the server will be exposed.</p>
    </div>

    <!-- Mode Selection (when limit is enabled) -->
    <div v-if="limitEnabled" class="mb-4 pl-4 border-l-2 border-gray-700">
      <!-- Radio: Allow only -->
      <label class="flex items-center gap-2 cursor-pointer mb-2">
        <input 
          type="radio" 
          name="filterMode" 
          value="allowlist" 
          v-model="filterMode"
          class="w-4 h-4 text-green-500 bg-background-darker border-gray-600 focus:ring-green-500"
        />
        <span :class="['text-sm', filterMode === 'allowlist' ? 'text-green-400' : 'text-gray-400']">
          Allow only selected
        </span>
        <span class="text-gray-500 text-xs">(more restrictive)</span>
      </label>
      
      <!-- Radio: Block selected -->
      <label class="flex items-center gap-2 cursor-pointer">
        <input 
          type="radio" 
          name="filterMode" 
          value="blocklist" 
          v-model="filterMode"
          class="w-4 h-4 text-red-500 bg-background-darker border-gray-600 focus:ring-red-500"
        />
        <span :class="['text-sm', filterMode === 'blocklist' ? 'text-red-400' : 'text-gray-400']">
          Block selected
        </span>
        <span class="text-gray-500 text-xs">(less restrictive)</span>
      </label>
    </div>

    <!-- Tools Selection (when limit is enabled) -->
    <div v-if="limitEnabled" class="mb-4">
      <label class="text-sm text-gray-400 mb-2 block">
        {{ filterMode === 'allowlist' ? 'Select tools to allow:' : 'Select tools to block:' }}
      </label>
      
      <!-- Available Tools -->
      <div v-if="toolsList.length > 0" class="mb-3">
        <div class="flex flex-wrap gap-2">
          <button 
            v-for="tool in toolsList" 
            :key="tool.name"
            @click="toggleTool(tool.name)"
            :class="[
              'px-2 py-1 rounded text-sm transition-all',
              isToolSelected(tool.name) 
                ? (filterMode === 'allowlist' ? 'bg-green-500/30 text-green-300 ring-1 ring-green-500' : 'bg-red-500/30 text-red-300 ring-1 ring-red-500')
                : 'bg-background-darker text-gray-400 hover:text-white'
            ]"
          >
            {{ tool.name }}
          </button>
        </div>
      </div>
      <div v-else class="mb-3 p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
        <p class="text-yellow-400 text-sm">Run "Test Connection" above to load available tools.</p>
      </div>
      
      <!-- Manual Input -->
      <div class="flex gap-2">
        <input 
          v-model="newTool" 
          @keyup.enter="addTool"
          type="text" 
          placeholder="Or type tool name manually"
          class="flex-1 bg-background-darker border border-gray-700 rounded-lg px-3 py-2 text-sm focus:border-primary focus:outline-none"
        />
        <button @click="addTool" class="btn btn-secondary text-sm">Add</button>
      </div>
      
      <!-- Selected Tools -->
      <div v-if="selectedTools.length > 0" class="mt-3 flex flex-wrap gap-2">
        <span 
          v-for="tool in selectedTools" 
          :key="tool" 
          :class="['px-2 py-1 rounded text-sm flex items-center gap-1', filterMode === 'allowlist' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400']"
        >
          {{ tool }}
          <button @click="removeTool(tool)" class="hover:text-white">
            <XMarkIcon class="w-4 h-4" />
          </button>
        </span>
      </div>
    </div>

    <!-- Summary -->
    <div v-if="summaryText" :class="['mb-4 p-3 rounded-lg', summaryClass]">
      <p :class="summaryTextClass">{{ summaryText }}</p>
    </div>

    <!-- Save Button -->
    <button 
      @click="handleSave" 
      :disabled="saving"
      class="btn btn-primary flex items-center gap-2"
    >
      <CheckIcon class="w-4 h-4" />
      {{ saving ? 'Saving...' : 'Save Rules' }}
    </button>

    <!-- Messages -->
    <p v-if="error" class="text-red-400 text-sm mt-3">{{ error }}</p>
    <p v-if="success" class="text-green-400 text-sm mt-3">{{ success }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import api from '@/api/client'
import { FunnelIcon, XMarkIcon, CheckIcon } from '@heroicons/vue/24/outline'

interface ToolInfo {
  name: string
  description?: string | null
}

const props = defineProps<{
  serverName: string
  availableTools?: ToolInfo[]
}>()

interface GovernanceConfig {
  server_name: string
  allowed_tools: string[]
  denied_tools: string[]
  tool_prefix: string
}

const limitEnabled = ref(false)
const filterMode = ref<'allowlist' | 'blocklist'>('allowlist')
const selectedTools = ref<string[]>([])
const prefix = ref('')
const newTool = ref('')
const saving = ref(false)
const error = ref<string | null>(null)
const success = ref<string | null>(null)
const toolsList = ref<ToolInfo[]>([])
const isLoading = ref(false)

const hasConfig = computed(() => {
  return selectedTools.value.length > 0 || prefix.value.length > 0
})

const summaryText = computed(() => {
  if (!limitEnabled.value && prefix.value) {
    return `All tools will be exposed with prefix "${prefix.value}_"`
  }
  if (limitEnabled.value && selectedTools.value.length > 0) {
    if (filterMode.value === 'allowlist') {
      return `Only ${selectedTools.value.length} tool(s) will be exposed${prefix.value ? ` with prefix "${prefix.value}_"` : ''}`
    } else {
      return `${selectedTools.value.length} tool(s) will be hidden${prefix.value ? `, rest will have prefix "${prefix.value}_"` : ''}`
    }
  }
  return ''
})

const summaryClass = computed(() => {
  if (!limitEnabled.value) return 'bg-blue-500/10 border border-blue-500/30'
  return filterMode.value === 'allowlist' 
    ? 'bg-green-500/10 border border-green-500/30' 
    : 'bg-red-500/10 border border-red-500/30'
})

const summaryTextClass = computed(() => {
  if (!limitEnabled.value) return 'text-blue-400 text-sm'
  return filterMode.value === 'allowlist' ? 'text-green-400 text-sm' : 'text-red-400 text-sm'
})

watch(filterMode, () => {
  // Don't clear when loading config from server
  if (!isLoading.value) {
    selectedTools.value = []
  }
})

watch(() => props.availableTools, (newTools) => {
  if (newTools) toolsList.value = newTools
}, { immediate: true })

onMounted(async () => {
  await loadConfig()
})

async function loadConfig() {
  isLoading.value = true
  try {
    const response = await api.get<GovernanceConfig>(`/servers/${props.serverName}/governance`)
    const config = response.data
    
    prefix.value = config.tool_prefix || ''
    
    if (config.allowed_tools.length > 0) {
      // Set selectedTools BEFORE filterMode to avoid watcher clearing them
      selectedTools.value = config.allowed_tools
      filterMode.value = 'allowlist'
      limitEnabled.value = true
    } else if (config.denied_tools.length > 0) {
      // Set selectedTools BEFORE filterMode to avoid watcher clearing them
      selectedTools.value = config.denied_tools
      filterMode.value = 'blocklist'
      limitEnabled.value = true
    } else {
      limitEnabled.value = false
      selectedTools.value = []
    }
  } catch {
    limitEnabled.value = false
    selectedTools.value = []
    prefix.value = ''
  } finally {
    isLoading.value = false
  }
}

function isToolSelected(toolName: string): boolean {
  return selectedTools.value.includes(toolName)
}

function toggleTool(toolName: string) {
  if (isToolSelected(toolName)) {
    selectedTools.value = selectedTools.value.filter(t => t !== toolName)
  } else {
    selectedTools.value.push(toolName)
  }
}

function addTool() {
  const tool = newTool.value.trim()
  if (tool && !selectedTools.value.includes(tool)) {
    selectedTools.value.push(tool)
  }
  newTool.value = ''
}

function removeTool(tool: string) {
  selectedTools.value = selectedTools.value.filter(t => t !== tool)
}

async function handleSave() {
  saving.value = true
  error.value = null
  success.value = null
  
  try {
    const payload = {
      allowed_tools: limitEnabled.value && filterMode.value === 'allowlist' ? selectedTools.value : [],
      denied_tools: limitEnabled.value && filterMode.value === 'blocklist' ? selectedTools.value : [],
      tool_prefix: prefix.value.trim()
    }
    
    await api.post(`/servers/${props.serverName}/governance`, payload)
    success.value = 'Governance rules saved!'
    setTimeout(() => { success.value = null }, 3000)
  } catch (e: unknown) {
    const err = e as { response?: { data?: string } }
    error.value = err.response?.data || 'Failed to save governance rules'
  } finally {
    saving.value = false
  }
}

async function handleClear() {
  if (!confirm('Clear all governance rules?')) return
  
  saving.value = true
  error.value = null
  
  try {
    await api.delete(`/servers/${props.serverName}/governance`)
    limitEnabled.value = false
    selectedTools.value = []
    prefix.value = ''
    success.value = 'Governance rules cleared!'
    setTimeout(() => { success.value = null }, 3000)
  } catch (e: unknown) {
    const err = e as { response?: { data?: string } }
    error.value = err.response?.data || 'Failed to clear rules'
  } finally {
    saving.value = false
  }
}
</script>
