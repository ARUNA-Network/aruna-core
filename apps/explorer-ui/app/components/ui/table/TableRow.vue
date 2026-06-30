<script setup lang="ts">
import { cn } from '~/utils/cn'

const props = defineProps<{
  class?: string
}>()
</script>

<template>
  <tr :class="cn('border-b border-border transition-colors hover:bg-bg-hover/50 data-[state=selected]:bg-muted', props.class)">
    <slot />
  </tr>
</template>
