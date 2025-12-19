<template>
  <div class="relative inline-block">
    <button 
      type="button"
      @mouseenter="show = true"
      @mouseleave="show = false"
      @focus="show = true"
      @blur="show = false"
      class="text-gray-400 hover:text-gray-300 focus:outline-none"
    >
      <InformationCircleIcon class="w-5 h-5" />
    </button>
    <Transition
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <div 
        v-if="show"
        class="absolute z-[100] w-80 p-4 mt-2 rounded-lg bg-gray-800 border border-gray-700 shadow-xl left-0"
      >
        <div class="flex items-start gap-3">
          <div :class="iconColorClass" class="shrink-0 mt-0.5">
            <component :is="typeIcon" class="w-5 h-5" />
          </div>
          <div>
            <h4 class="font-medium text-white mb-1">{{ title }}</h4>
            <p class="text-sm text-gray-400 leading-relaxed">{{ content }}</p>
            <slot></slot>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { 
  InformationCircleIcon,
  LightBulbIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon
} from '@heroicons/vue/24/outline'

const props = withDefaults(defineProps<{
  title: string
  content: string
  type?: 'info' | 'tip' | 'warning' | 'success'
  position?: 'left' | 'right'
}>(), {
  type: 'info',
  position: 'left'
})

const show = ref(false)

const typeIcon = computed(() => {
  switch (props.type) {
    case 'tip': return LightBulbIcon
    case 'warning': return ExclamationTriangleIcon
    case 'success': return CheckCircleIcon
    default: return InformationCircleIcon
  }
})

const iconColorClass = computed(() => {
  switch (props.type) {
    case 'tip': return 'text-yellow-400'
    case 'warning': return 'text-orange-400'
    case 'success': return 'text-green-400'
    default: return 'text-blue-400'
  }
})
</script>
