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
          @click="mode = 'whitelist'" 
          :class="['px-3 py-1.5 rounded-lg text-sm transition-colors', mode === 'whitelist' ? 'bg-green-600 text-white' : 'bg-background-darker text-gray-400 hover:text-white']"
        >
          Whitelist
        </button>
        <button 
          @click="mode = 'blacklist'" 
          :class="['px-3 py-1.5 rounded-lg text-sm transition-colors', mode === 'blacklist' ? 'bg-red-600 text-white' : 'bg-background-darker text-gray-400 hover:text-white']"
        >
          Blacklist
        </button>
      </div>
    </div>

    <!-- Tools Selection (for whitelist/blacklist) -->
    <div v-if="mode !== 'none'" class="mb-4">
      <label class="text-sm text-gray-400 mb-2 block">
        {{ mode === 'whitelist' ? 'Allowed Tools (only these will be exposed)' : 'Blocked Tools (these will be hidden)' }}
      </label>
      
      <!-- Available Tools from Server (clickable chips) -->
      <div v-if="availableTools.length > 0" class="mb-3">
        <p class="text-xs text-gray-500 mb-2">Click to {{ mode === 'whitelist' ? 'allow' : 'block' }}:</p>
        <div class="flex flex-wrap gap-2">
          <button 
            v-for="tool in availableTools" 
            :key="tool.name"
            @click="toggleTool(tool.name)"
            :class="[
              'px-2 py-1 rounded text-sm transition-all',
              isToolSelected(tool.name) 
                ? (mode === 'whitelist' ? 'bg-green-500/30 text-green-300 ring-1 ring-green-500' : 'bg-red-500/30 text-red-300 ring-1 ring-red-500')
                : 'bg-background-darker text-gray-400 hover:text-white'
            ]"
          >
            {{ tool.name }}
          </button>
        </div>
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
      <div v-if="tools.length > 0" class="mt-3">
        <p class="text-xs text-gray-500 mb-2">{{ mode === 'whitelist' ? 'Allowed' : 'Blocked' }} tools:</p>
        <div class="flex flex-wrap gap-2">
          <span 
            v-for="tool in tools" 
            :key="tool" 
            :class="['px-2 py-1 rounded text-sm flex items-center gap-1', mode === 'whitelist' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400']"
          >
            {{ tool }}
            <button @click="removeTool(tool)" class="hover:text-white">
              <XMarkIcon class="w-4 h-4" />
            </button>
          </span>
        </div>
      </div>
      <p v-else class="text-gray-500 text-sm">No tools selected</p>
    </div>

    <!-- Summary -->
    <div v-if="mode === 'none' && prefix" class="mb-4 p-3 bg-blue-500/10 border border-blue-500/30 rounded-lg">
      <p class="text-blue-400 text-sm">All tools will be exposed with prefix "{{ prefix }}_"</p>
    </div>
    <div v-else-if="mode === 'whitelist' && tools.length > 0" class="mb-4 p-3 bg-green-500/10 border border-green-500/30 rounded-lg">
      <p class="text-green-400 text-sm">Only {{ tools.length }} tool(s) will be exposed{{ prefix ? ` with prefix "${prefix}_"` : '' }}</p>
    </div>
    <div v-else-if="mode === 'blacklist' && tools.length > 0" class="mb-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
      <p class="text-red-400 text-sm">{{ tools.length }} tool(s) will be hidden{{ prefix ? `, rest will have prefix "${prefix}_"` : '' }}</p>
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

const mode = ref<'none' | 'whitelist' | 'blacklist'>('none')
const tools = ref<string[]>([])
const prefix = ref('')
const newTool = ref('')
const saving = ref(false)
const error = ref<string | null>(null)
const success = ref<string | null>(null)
const availableTools = ref<ToolInfo[]>([])

const hasConfig = computed(() => {
  return tools.value.length > 0 || prefix.value.length > 0
})

// Watch mode changes to clear tools when switching
watch(mode, (newMode, oldMode) => {
  if (newMode !== oldMode && newMode === 'none') {
    tools.value = []
  }
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
    
    if (config.allowed_tools.length > 0) {
      mode.value = 'whitelist'
      tools.value = config.allowed_tools
    } else if (config.denied_tools.length > 0) {
      mode.value = 'blacklist'
      tools.value = config.denied_tools
    } else {
      mode.value = 'none'
      tools.value = []
    }
  } catch (e) {
    mode.value = 'none'
    tools.value = []
    prefix.value = ''
  }
}

function isToolSelected(toolName: string): boolean {
  return tools.value.includes(toolName)
}

function toggleTool(toolName: string) {
  if (isToolSelected(toolName)) {
    removeTool(toolName)
  } else {
    tools.value.push(toolName)
  }
}

function addTool() {
  const tool = newTool.value.trim()
  if (tool && !tools.value.includes(tool)) {
    tools.value.push(tool)
    newTool.value = ''
  }
}

function removeTool(tool: string) {
  tools.value = tools.value.filter(t => t !== tool)
}

async function handleSave() {
  saving.value = true
  error.value = null
  success.value = null
  
  try {
    const payload = {
      allowed_tools: mode.value === 'whitelist' ? tools.value : [],
      denied_tools: mode.value === 'blacklist' ? tools.value : [],
      tool_prefix: prefix.value.trim()
    }
    
    await api.post(`/servers/${props.serverName}/governance`, payload)
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
    tools.value = []
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
