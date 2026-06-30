<script setup lang="ts">
import { ref } from 'vue'
import Sidebar from '~/components/Sidebar.vue'
import Header from '~/components/Header.vue'
import Footer from '~/components/Footer.vue'

const isSidebarOpen = ref(false)
</script>

<template>
  <div class="app-layout min-h-screen flex text-foreground bg-background">
    <!-- Left Sidebar -->
    <Sidebar :is-open="isSidebarOpen" @close="isSidebarOpen = false" />

    <!-- Main Right Content Wrapper -->
    <div class="flex-grow flex flex-col min-w-0 lg:pl-64">
      <!-- Top Header / Navbar -->
      <Header :is-sidebar-open="isSidebarOpen" @toggle-sidebar="isSidebarOpen = !isSidebarOpen" />

      <!-- Inner View Slot -->
      <div class="flex-grow">
        <slot />
      </div>

      <!-- Bottom Footer -->
      <Footer />
    </div>
  </div>
</template>

<style scoped>
.app-layout {
  background-color: var(--bg-base);
}
</style>
