<template>
  <Teleport to="body">
    <div v-if="isOpen" class="modal-overlay" @click.self="handleBackdropClick">
      <div class="modal-content">
        <div class="modal-header">
          <h2 class="text-xl font-bold flex items-center gap-2">
            <ServerIcon class="w-6 h-6 text-primary" />
            Configurando "{{ serverName }}"
          </h2>
        </div>
        
        <div class="modal-body">
          <!-- Progress Steps -->
          <div class="space-y-4">
            <!-- Step 1: Save Server -->
            <div class="step" :class="stepClass(1)">
              <div class="step-icon">
                <CheckCircleIcon v-if="currentStep > 1" class="w-5 h-5 text-green-400" />
                <ArrowPathIcon v-else-if="currentStep === 1" class="w-5 h-5 text-primary animate-spin" />
                <div v-else class="w-5 h-5 rounded-full border-2 border-gray-600" />
              </div>
              <span class="step-text">Salvando servidor no banco</span>
            </div>
            
            <!-- Step 2: Test Connectivity -->
            <div class="step" :class="stepClass(2)">
              <div class="step-icon">
                <CheckCircleIcon v-if="currentStep > 2" class="w-5 h-5 text-green-400" />
                <ArrowPathIcon v-else-if="currentStep === 2" class="w-5 h-5 text-primary animate-spin" />
                <div v-else class="w-5 h-5 rounded-full border-2 border-gray-600" />
              </div>
              <span class="step-text">Testando conectividade</span>
            </div>
            
            <!-- Step 3: Auth (conditional) -->
            <div v-if="requiresAuth" class="step" :class="stepClass(3)">
              <div class="step-icon">
                <CheckCircleIcon v-if="currentStep > 3" class="w-5 h-5 text-green-400" />
                <ExclamationTriangleIcon v-else-if="currentStep === 3 && !authStarted" class="w-5 h-5 text-yellow-400" />
                <ArrowPathIcon v-else-if="currentStep === 3 && authStarted" class="w-5 h-5 text-primary animate-spin" />
                <div v-else class="w-5 h-5 rounded-full border-2 border-gray-600" />
              </div>
              <span class="step-text">
                {{ currentStep === 3 && !authStarted ? 'Autorização OAuth necessária' : 'Autenticação OAuth' }}
              </span>
            </div>
            
            <!-- Step 4: Load Tools -->
            <div class="step" :class="stepClass(requiresAuth ? 4 : 3)">
              <div class="step-icon">
                <CheckCircleIcon v-if="toolsLoaded" class="w-5 h-5 text-green-400" />
                <ArrowPathIcon v-else-if="loadingTools" class="w-5 h-5 text-primary animate-spin" />
                <div v-else class="w-5 h-5 rounded-full border-2 border-gray-600" />
              </div>
              <span class="step-text">
                {{ toolsLoaded ? `Tools carregadas (${toolsCount})` : 'Carregando tools' }}
              </span>
            </div>
          </div>
          
          <!-- Auth Required Alert -->
          <div v-if="currentStep === 3 && !authStarted && requiresAuth && !skippedAuth" class="mt-6 p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
            <p class="text-yellow-400 text-sm mb-3">
              Este servidor requer autorização OAuth. Clique abaixo para autorizar.
            </p>
            <div class="flex gap-2">
              <button @click="startAuth" class="btn btn-primary">
                <LockOpenIcon class="w-4 h-4" />
                Autorizar Agora
              </button>
              <button @click="skipAuth" class="btn btn-ghost text-yellow-400">
                Fazer Depois
              </button>
            </div>
          </div>
          
          <!-- Error State (hidden during OAuth step) -->
          <div v-if="error && currentStep !== 3" class="mt-6 p-4 bg-red-500/10 border border-red-500/30 rounded-lg">
            <p class="text-red-400 text-sm mb-3">{{ error }}</p>
            <div class="flex gap-2">
              <button @click="retry" class="btn btn-ghost text-sm">Tentar Novamente</button>
              <button @click="goToServer" class="btn btn-primary text-sm">Ir para Servidor</button>
            </div>
          </div>
          
          <!-- Success State -->
          <div v-if="isComplete" class="mt-6 p-4 bg-green-500/10 border border-green-500/30 rounded-lg">
            <p class="text-green-400 text-sm mb-3">
              ✓ Servidor configurado com sucesso!
            </p>
            <button @click="goToServer" class="btn btn-primary">
              Ir para Servidor
            </button>
          </div>
          
          <!-- Pending Auth State (skipped OAuth) -->
          <div v-if="skippedAuth" class="mt-6 p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
            <p class="text-yellow-400 text-sm mb-3">
              ⚠ Servidor cadastrado com pendências. Autorize o OAuth na página do servidor.
            </p>
            <button @click="goToServer" class="btn btn-primary">
              Ir para Servidor
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { 
  ServerIcon, 
  CheckCircleIcon, 
  ArrowPathIcon,
  ExclamationTriangleIcon,
  LockOpenIcon
} from '@heroicons/vue/24/outline'
import api from '@/api/client'
import { startOAuthFlow } from '@/lib/mcp'

const props = defineProps<{
  isOpen: boolean
  serverName: string
  serverUrl: string
  authType: string
  formData: Record<string, unknown>
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'complete', serverName: string): void
}>()

const router = useRouter()

const currentStep = ref(0)
const authStarted = ref(false)
const toolsLoaded = ref(false)
const loadingTools = ref(false)
const toolsCount = ref(0)
const error = ref<string | null>(null)
const createdServerName = ref('')
const skippedAuth = ref(false)

const requiresAuth = computed(() => props.authType === 'oauth_auto')
const isComplete = computed(() => toolsLoaded.value && !error.value && !skippedAuth.value)

function stepClass(step: number) {
  const effectiveStep = currentStep.value
  // Show OAuth step (3) as skipped/yellow when skipped
  if (skippedAuth.value && step === 3) return 'step-skipped'
  if (effectiveStep > step) return 'step-complete'
  if (effectiveStep === step) return 'step-active'
  return 'step-pending'
}

watch(() => props.isOpen, (open) => {
  if (open) {
    startSetup()
  } else {
    resetState()
  }
})

function resetState() {
  currentStep.value = 0
  authStarted.value = false
  toolsLoaded.value = false
  loadingTools.value = false
  toolsCount.value = 0
  error.value = null
  skippedAuth.value = false
}

function skipAuth() {
  // Mark OAuth as skipped - server will be in pending_auth status
  skippedAuth.value = true
}

async function startSetup() {
  resetState()
  
  try {

    currentStep.value = 1
    const response = await api.post('/servers', props.formData)
    createdServerName.value = response.data.name
    

    currentStep.value = 2
    await testConnectivity()
    
  } catch (e: unknown) {
    const errMsg = e instanceof Error ? e.message : 'Erro ao criar servidor'
    error.value = errMsg
  }
}

async function testConnectivity() {
  try {
    const testResult = await api.post(`/servers/${createdServerName.value}/test`)
    
    if (testResult.data.success) {
  
      toolsLoaded.value = true
      toolsCount.value = testResult.data.tools?.length || 0
      currentStep.value = requiresAuth.value ? 5 : 4
    } else {
      const msg = testResult.data.message || ''
      const needsAuth = msg.includes('401') || msg.includes('AuthRequired') || msg.includes('oauth')
      
      if (requiresAuth.value && needsAuth) {
    
        error.value = null
        currentStep.value = 3
      } else {
        error.value = msg || 'Falha na conexão'
      }
    }
  } catch (e: unknown) {
    if (requiresAuth.value) {
  
      error.value = null
      currentStep.value = 3
    } else {
      const errMsg = e instanceof Error ? e.message : 'Erro ao testar conexão'
      error.value = errMsg
    }
  }
}

async function startAuth() {
  authStarted.value = true
  
  try {

    const serverResponse = await api.get(`/servers/${createdServerName.value}`)
    const server = serverResponse.data
    

    const result = await startOAuthFlow(createdServerName.value, server.url)
    
    if (result.success) {
  
  
      const maxAttempts = 60 // 60 seconds max wait
      let attempts = 0
      let oauthComplete = false
      
      while (attempts < maxAttempts && !oauthComplete) {
        await new Promise(resolve => setTimeout(resolve, 1000)) // Wait 1 second
        attempts++
        
        try {
          const statusResponse = await api.get(`/servers/${createdServerName.value}/oauth/status`)
          if (statusResponse.data.connected) {
            oauthComplete = true
          }
        } catch {
          // Ignore OAuth status check errors
        }
      }
      
      if (!oauthComplete) {
        error.value = 'OAuth flow timeout - popup may have been closed'
        authStarted.value = false
        return
      }
      
  
      currentStep.value = requiresAuth.value ? 4 : 3
      loadingTools.value = true
      
      const testResult = await api.post(`/servers/${createdServerName.value}/test`)
      loadingTools.value = false
      
      if (testResult.data.success) {
        toolsLoaded.value = true
        toolsCount.value = testResult.data.tools?.length || 0
        currentStep.value = 5
      } else {
        error.value = testResult.data.message || 'Falha ao carregar tools'
      }
    } else {
      error.value = result.error || 'Falha na autorização OAuth'
    }
  } catch (e: unknown) {
    authStarted.value = false
    const errMsg = e instanceof Error ? e.message : 'Falha na autorização OAuth'
    error.value = errMsg
  }
}

function retry() {
  error.value = null
  if (currentStep.value === 3) {
    startAuth()
  } else {
    startSetup()
  }
}

function goToServer() {
  emit('complete', createdServerName.value || props.serverName)
  router.push(`/servers/${createdServerName.value || props.serverName}`)
}

function handleBackdropClick() {
  if (isComplete.value || error.value) {
    emit('close')
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
  backdrop-filter: blur(4px);
}

.modal-content {
  background: #1a1a2e;
  border-radius: 12px;
  border: 2px solid var(--color-primary);
  box-shadow: 0 0 30px rgba(99, 102, 241, 0.3), 0 20px 60px rgba(0, 0, 0, 0.5);
  width: 100%;
  max-width: 480px;
  margin: 1rem;
}

.modal-header {
  padding: 1.5rem;
  border-bottom: 1px solid var(--color-border);
}

.modal-body {
  padding: 1.5rem;
}

.step {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  border-radius: 8px;
  transition: all 0.2s;
}

.step-icon {
  flex-shrink: 0;
}

.step-text {
  font-size: 0.9rem;
}

.step-pending {
  opacity: 0.5;
}

.step-active {
  background: var(--color-primary-dark);
  opacity: 1;
}

.step-complete {
  opacity: 1;
}

.step-skipped {
  background: rgba(245, 158, 11, 0.1);
  opacity: 1;
}
</style>
