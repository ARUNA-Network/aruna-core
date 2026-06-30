<script setup lang="ts">
import { cn } from '~/utils/cn'

const props = defineProps<{
  class?: string
}>()
</script>

<template>
  <div :class="cn('flex flex-col space-y-1.5 p-6 border-b border-border/40', props.class)">
    <slot />
  </div>
</template>
