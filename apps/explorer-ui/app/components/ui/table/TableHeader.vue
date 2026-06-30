<script setup lang="ts">
import { cn } from '~/utils/cn'

const props = defineProps<{
  class?: string
}>()
</script>

<template>
  <thead :class="cn('[&_tr]:border-b border-border/60 bg-bg-elevated/40', props.class)">
    <slot />
  </thead>
</template>
