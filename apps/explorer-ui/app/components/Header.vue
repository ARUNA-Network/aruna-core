<script setup lang="ts">
import Navbar from '~/components/Navbar.vue'

defineProps<{
  isSidebarOpen: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle-sidebar'): void
}>()
</script>

<template>
  <header>
    <Navbar :is-sidebar-open="isSidebarOpen" @toggle-sidebar="emit('toggle-sidebar')" />
  </header>
</template>
