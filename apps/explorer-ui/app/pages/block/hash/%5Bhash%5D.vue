<script setup lang="ts">
import { useRoute, navigateTo } from '#app'

const route = useRoute()
// Redirect `/block/hash/:hash` to `/block/:hash`
navigateTo(`/block/${route.params.hash}`, { replace: true })
</script>

<template>
  <main class="container page-spacing">
    <div class="empty-state">Redirecting...</div>
  </main>
</template>
