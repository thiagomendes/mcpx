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

    <!-- Prefix Section (applies to ALL tools) -->
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
      <p class="text-gray-500 text-xs mt-1">Alphanumeric + underscore only. Leave empty for no prefix.</p>
    </div>
    
    <!-- Filter Mode Selection -->
    <div class="mb-4">
      <label class="text-sm text-gray-400 mb-2 block">Tool Filter</label>
      <div class="flex gap-2">
        <button 
          @click="mode = 'none'" 
          :class="['px-3 py-1.5 rounded-lg text-sm transition-colors', mode === 'none' ? 'bg-primary text-white' : 'bg-background-darker text-gray-400 hover:text-white']"
        >
          All Tools
        </button>
        <button 
          @click="mode = 'allowlist'" 
          :class="['px-3 py-1.5 rounded-lg text-sm transition-colors', mode === 'allowlist' ? 'bg-green-600 text-white' : 'bg-background-darker text-gray-400 hover:text-white']"
        >
          Allowlist
          <span v-if="allowedTools.length > 0" class="ml-1 text-xs">({{ allowedTools.length }})</span>
        </button>
        <button 
          @click="mode = 'blocklist'" 
          :class="['px-3 py-1.5 rounded-lg text-sm transition-colors', mode === 'blocklist' ? 'bg-red-600 text-white' : 'bg-background-darker text-gray-400 hover:text-white']"
        >
          Blocklist
          <span v-if="blockedTools.length > 0" class="ml-1 text-xs">({{ blockedTools.length }})</span>
        </button>
      </div>
      <p class="text-gray-500 text-xs mt-2">
        <span v-if="mode === 'none'">All tools from the server will be exposed.</span>
        <span v-else-if="mode === 'allowlist'">Only selected tools will be exposed (others hidden).</span>
        <span v-else>Selected tools will be hidden (others exposed).</span>
      </p>
    </div>

    <!-- Tools Selection (for allowlist/blocklist) -->
    <div v-if="mode !== 'none'" class="mb-4">
      <label class="text-sm text-gray-400 mb-2 block">
        {{ mode === 'allowlist' ? 'Select tools to allow:' : 'Select tools to block:' }}
      </label>
      
      <!-- Available Tools from Server (clickable chips) -->
      <div v-if="availableTools.length > 0" class="mb-3">
        <div class="flex flex-wrap gap-2">
          <button 
            v-for="tool in availableTools" 
            :key="tool.name"
            @click="toggleTool(tool.name)"
            :class="[
              'px-2 py-1 rounded text-sm transition-all',
              isToolSelected(tool.name) 
                ? (mode === 'allowlist' ? 'bg-green-500/30 text-green-300 ring-1 ring-green-500' : 'bg-red-500/30 text-red-300 ring-1 ring-red-500')
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
      <div class="flex gap-2 mb-2">
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
      <div v-if="currentTools.length > 0" class="mt-3">
        <p class="text-xs text-gray-500 mb-2">{{ mode === 'allowlist' ? 'Allowed' : 'Blocked' }} tools:</p>
        <div class="flex flex-wrap gap-2">
          <span 
            v-for="tool in currentTools" 
            :key="tool" 
            :class="['px-2 py-1 rounded text-sm flex items-center gap-1', mode === 'allowlist' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400']"
          >
            {{ tool }}
            <button @click="removeTool(tool)" class="hover:text-white">
              <XMarkIcon class="w-4 h-4" />
            </button>
          </span>
        </div>
      </div>
    </div>

    <!-- Summary -->
    <div v-if="mode === 'none' && prefix" class="mb-4 p-3 bg-blue-500/10 border border-blue-500/30 rounded-lg">
      <p class="text-blue-400 text-sm">All tools will be exposed with prefix "{{ prefix }}_"</p>
    </div>
    <div v-else-if="mode === 'allowlist' && allowedTools.length > 0" class="mb-4 p-3 bg-green-500/10 border border-green-500/30 rounded-lg">
      <p class="text-green-400 text-sm">Only {{ allowedTools.length }} tool(s) will be exposed{{ prefix ? ` with prefix "${prefix}_"` : '' }}</p>
    </div>
    <div v-else-if="mode === 'blocklist' && blockedTools.length > 0" class="mb-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
      <p class="text-red-400 text-sm">{{ blockedTools.length }} tool(s) will be hidden{{ prefix ? `, rest will have prefix "${prefix}_"` : '' }}</p>
    </div>

    <!-- Save Button -->
    <div class="flex gap-2">
      <button 
        @click="handleSave" 
        :disabled="saving"
        class="btn btn-primary flex items-center gap-2"
      >
        <CheckIcon class="w-4 h-4" />
        {{ saving ? 'Saving...' : 'Save Rules' }}
      </button>
    </div>

    <!-- Error/Success Messages -->
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
  description?: string
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

const mode = ref<'none' | 'allowlist' | 'blocklist'>('none')
const allowedTools = ref<string[]>([])  // Separate array for allowlist
const blockedTools = ref<string[]>([])  // Separate array for blocklist
const prefix = ref('')
const newTool = ref('')
const saving = ref(false)
const error = ref<string | null>(null)
const success = ref<string | null>(null)
const availableTools = ref<ToolInfo[]>([])

// Get current tools array based on mode
const currentTools = computed(() => {
  return mode.value === 'allowlist' ? allowedTools.value : blockedTools.value
})

const hasConfig = computed(() => {
  return allowedTools.value.length > 0 || blockedTools.value.length > 0 || prefix.value.length > 0
})

// Watch props for available tools
watch(() => props.availableTools, (newTools) => {
  if (newTools) {
    availableTools.value = newTools
  }
}, { immediate: true })

onMounted(async () => {
  await loadConfig()
})

async function loadConfig() {
  try {
    const response = await api.get<GovernanceConfig>(`/servers/${props.serverName}/governance`)
    const config = response.data
    
    prefix.value = config.tool_prefix || ''
    allowedTools.value = config.allowed_tools || []
    blockedTools.value = config.denied_tools || []
    
    if (allowedTools.value.length > 0) {
      mode.value = 'allowlist'
    } else if (blockedTools.value.length > 0) {
      mode.value = 'blocklist'
    } else {
      mode.value = 'none'
    }
  } catch (e) {
    mode.value = 'none'
    allowedTools.value = []
    blockedTools.value = []
    prefix.value = ''
  }
}

function isToolSelected(toolName: string): boolean {
  return currentTools.value.includes(toolName)
}

function toggleTool(toolName: string) {
  const arr = mode.value === 'allowlist' ? allowedTools : blockedTools
  if (arr.value.includes(toolName)) {
    arr.value = arr.value.filter(t => t !== toolName)
  } else {
    arr.value.push(toolName)
  }
}

function addTool() {
  const tool = newTool.value.trim()
  if (!tool) return
  
  const arr = mode.value === 'allowlist' ? allowedTools : blockedTools
  if (!arr.value.includes(tool)) {
    arr.value.push(tool)
  }
  newTool.value = ''
}

function removeTool(tool: string) {
  const arr = mode.value === 'allowlist' ? allowedTools : blockedTools
  arr.value = arr.value.filter(t => t !== tool)
}

async function handleSave() {
  saving.value = true
  error.value = null
  success.value = null
  
  try {
    // Based on mode, send appropriate tools
    const payload = {
      allowed_tools: mode.value === 'allowlist' ? allowedTools.value : [],
      denied_tools: mode.value === 'blocklist' ? blockedTools.value : [],
      tool_prefix: prefix.value.trim()
    }
    
    // If mode is 'none', clear both lists
    if (mode.value === 'none') {
      payload.allowed_tools = []
      payload.denied_tools = []
    }
    
    await api.post(`/servers/${props.serverName}/governance`, payload)
    
    // Clear the list that's not active
    if (mode.value === 'allowlist') {
      blockedTools.value = []
    } else if (mode.value === 'blocklist') {
      allowedTools.value = []
    } else {
      allowedTools.value = []
      blockedTools.value = []
    }
    
    success.value = 'Governance rules saved!'
    setTimeout(() => { success.value = null }, 3000)
  } catch (e: any) {
    error.value = e.response?.data || 'Failed to save governance rules'
  } finally {
    saving.value = false
  }
}

async function handleClear() {
  if (!confirm('Clear all governance rules? All tools will be exposed without prefix.')) return
  
  saving.value = true
  error.value = null
  
  try {
    await api.delete(`/servers/${props.serverName}/governance`)
    mode.value = 'none'
    allowedTools.value = []
    blockedTools.value = []
    prefix.value = ''
    success.value = 'Governance rules cleared!'
    
    setTimeout(() => { success.value = null }, 3000)
  } catch (e: any) {
    error.value = e.response?.data || 'Failed to clear governance rules'
  } finally {
    saving.value = false
  }
}
</script>
