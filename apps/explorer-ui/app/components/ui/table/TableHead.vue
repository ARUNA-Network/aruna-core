<script setup lang="ts">
import { cn } from '~/utils/cn'

const props = defineProps<{
  class?: string
}>()
</script>

<template>
  <th :class="cn('h-12 px-4 text-left align-middle font-semibold text-text-secondary [&:has([role=checkbox])]:pr-0', props.class)">
    <slot />
  </th>
</template>
