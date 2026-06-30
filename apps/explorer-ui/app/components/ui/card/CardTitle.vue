<script setup lang="ts">
import { cn } from '~/utils/cn'

const props = defineProps<{
  class?: string
}>()
</script>

<template>
  <h3 :class="cn('text-lg font-bold leading-none tracking-tight flex items-center gap-2', props.class)">
    <slot />
  </h3>
</template>
