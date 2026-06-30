<script setup lang="ts">
import { useRoute } from '#app'
import SearchBar from '~/components/SearchBar.vue'
import Button from '~/components/ui/button/Button.vue'

defineProps<{
  isSidebarOpen: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle-sidebar'): void
}>()

const route = useRoute()
</script>

<template>
  <nav class="navbar w-full flex items-center justify-between px-6 h-16 border-b border-border bg-background/80 backdrop-blur-md sticky top-0 z-50">
    <div class="flex items-center gap-4">
      <!-- Mobile sidebar toggle -->
      <Button
        variant="ghost"
        size="icon"
        class="lg:hidden"
        @click="emit('toggle-sidebar')"
        aria-label="Toggle Sidebar"
      >
        <span class="text-xl">☰</span>
      </Button>

      <!-- Logo on Mobile -->
      <NuxtLink to="/" class="flex items-center gap-2 lg:hidden">
        <div class="logo-icon text-xl">⬡</div>
        <span class="logo-name font-extrabold text-sm text-glow">ARUNA</span>
      </NuxtLink>

      <!-- Breadcrumbs / Page context on desktop -->
      <div class="hidden lg:flex items-center gap-2 text-xs font-semibold text-text-secondary">
        <span>EXPLORER</span>
        <span class="text-text-muted">/</span>
        <span class="text-glow text-brand-primary uppercase">{{ route.name?.toString().replace('-id', '').replace('-height', '') || 'HOME' }}</span>
      </div>
    </div>

    <!-- SearchBar on sub-pages (always shown on navbar if not home page) -->
    <div class="w-64 md:w-80" v-if="route.path !== '/'">
      <SearchBar :small="true" placeholder="Search hash, height, address..." />
    </div>
  </nav>
</template>

<style scoped>
.navbar {
  background: hsla(222, 30%, 6%, 0.82);
  backdrop-filter: blur(20px) saturate(180%);
  border-bottom: 1px solid var(--border);
}
</style>
