<template>
  <DashboardLayout>
    <div class="max-w-4xl">
      <div class="mb-6">
        <h1 class="text-2xl font-bold mb-1">System Configuration</h1>
        <p class="text-gray-400">Configure token expiry and resource limits for your organization</p>
      </div>

      <!-- Loading State -->
      <div v-if="loading" class="card py-8 text-center text-gray-400">
        Loading settings...
      </div>

      <!-- Error State -->
      <div v-else-if="error" class="card py-8 text-center">
        <ExclamationTriangleIcon class="w-12 h-12 mx-auto text-amber-500 mb-3" />
        <p class="text-gray-400">{{ error }}</p>
        <button @click="fetchSettings" class="btn btn-primary mt-4">Retry</button>
      </div>

      <!-- Settings Content -->
      <div v-else>
        <!-- Read-only Notice for Members -->
        <div v-if="!canEdit" class="card bg-blue-500/10 border-blue-500/30 mb-6">
          <div class="flex items-center gap-3">
            <InformationCircleIcon class="w-5 h-5 text-blue-400" />
            <span class="text-gray-300">You have read-only access. Only organization owners and admins can modify settings.</span>
          </div>
        </div>

        <!-- Tabs -->
        <div class="flex gap-2 mb-6 border-b border-gray-700 pb-2">
          <button 
            v-for="tab in tabs" 
            :key="tab.key"
            :class="[
              'flex items-center gap-2 px-4 py-2 rounded-t transition-all',
              activeTab === tab.key 
                ? 'bg-gray-800 text-violet-400 border-b-2 border-violet-400' 
                : 'text-gray-400 hover:text-white hover:bg-gray-800/50'
            ]"
            @click="activeTab = tab.key"
          >
            <component :is="tab.icon" class="w-5 h-5" />
            {{ tab.label }}
          </button>
        </div>

        <!-- Tokens Tab -->
        <div v-if="activeTab === 'tokens'" class="card">
          <div class="flex items-start gap-3 p-4 bg-violet-500/10 rounded-lg mb-6">
            <ShieldCheckIcon class="w-5 h-5 text-violet-400 shrink-0 mt-0.5" />
            <span class="text-gray-300 text-sm">Token settings affect security and user experience. Shorter durations are more secure but require more frequent re-authentication.</span>
          </div>

          <div class="space-y-4">
            <SettingCard
              v-for="setting in getSettingsByCategory('tokens')"
              :key="setting.key"
              :setting="setting"
              :meta="getSettingMeta(setting.key)"
              :can-edit="canEdit"
              :saving="saving === setting.key"
              @update="handleUpdateSetting"
            />
          </div>
        </div>

        <!-- Limits Tab -->
        <div v-if="activeTab === 'limits'" class="card">
          <div class="flex items-start gap-3 p-4 bg-violet-500/10 rounded-lg mb-6">
            <ChartBarIcon class="w-5 h-5 text-violet-400 shrink-0 mt-0.5" />
            <span class="text-gray-300 text-sm">Resource limits help prevent abuse and ensure fair usage across the organization.</span>
          </div>

          <div class="space-y-4">
            <SettingCard
              v-for="setting in getSettingsByCategory('limits')"
              :key="setting.key"
              :setting="setting"
              :meta="getSettingMeta(setting.key)"
              :can-edit="canEdit"
              :saving="saving === setting.key"
              @update="handleUpdateSetting"
            />
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
    </div>
  </DashboardLayout>
</template>

<script setup lang="ts">
import { ref, onMounted, markRaw, type Component } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { storeToRefs } from 'pinia'
import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import SettingCard from '@/components/SettingCard.vue'
import { 
  ExclamationTriangleIcon, 
  InformationCircleIcon,
  ShieldCheckIcon,
  ChartBarIcon,
  ArrowLeftIcon
} from '@heroicons/vue/24/outline'

const settingsStore = useSettingsStore()
const { canEdit, loading, error, saving } = storeToRefs(settingsStore)
const { fetchSettings, updateSetting, getSettingMeta, getSettingsByCategory } = settingsStore

const activeTab = ref('tokens')

interface Tab {
  key: string
  label: string
  icon: Component
}

const tabs: Tab[] = [
  { key: 'tokens', label: 'Security', icon: markRaw(ShieldCheckIcon) },
  { key: 'limits', label: 'Limits', icon: markRaw(ChartBarIcon) },
]

async function handleUpdateSetting(key: string, value: string) {
  await updateSetting(key, value)
}

onMounted(() => {
  fetchSettings()
})
</script>
