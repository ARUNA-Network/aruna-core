<script setup lang="ts">
const config = useRuntimeConfig()

defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()
</script>

<template>
  <div>
    <!-- Backdrop for mobile -->
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/60 z-40 lg:hidden"
      @click="emit('close')"
    ></div>

    <!-- Sidebar container -->
    <aside
      :class="[
        'sidebar fixed top-0 bottom-0 left-0 z-50 w-64 border-r border-border bg-bg-elevated p-6 flex flex-col gap-6 transition-transform duration-300 lg:translate-x-0',
        isOpen ? 'translate-x-0' : '-translate-x-full'
      ]"
    >
      <!-- Logo header -->
      <NuxtLink to="/" class="flex items-center gap-3 py-2 border-b border-border/60" @click="emit('close')">
        <div class="logo-icon text-2xl">⬡</div>
        <div class="logo-text">
          <span class="logo-name font-extrabold tracking-wider text-base text-glow">ARUNA</span>
          <span class="logo-sub text-[9px] tracking-widest text-text-muted">EXPLORER</span>
        </div>
      </NuxtLink>

      <div class="px-2">
        <Badge variant="default" class="text-[10px] py-0.5 px-2">{{ config.public.networkName }}</Badge>
      </div>

      <!-- Navigation links -->
      <nav class="flex-1 flex flex-col gap-1.5" role="navigation">
        <NuxtLink
          to="/"
          class="sidebar-link"
          active-class="active"
          exact-active-class="active"
          @click="emit('close')"
        >
          <span class="link-icon">🏠</span> Home
        </NuxtLink>
        <NuxtLink
          to="/blocks"
          class="sidebar-link"
          active-class="active"
          @click="emit('close')"
        >
          <span class="link-icon">📦</span> Blocks
        </NuxtLink>
        <NuxtLink
          to="/transactions"
          class="sidebar-link"
          active-class="active"
          @click="emit('close')"
        >
          <span class="link-icon">⚡</span> Transactions
        </NuxtLink>
        <NuxtLink
          to="/network"
          class="sidebar-link"
          active-class="active"
          @click="emit('close')"
        >
          <span class="link-icon">🌐</span> Network
        </NuxtLink>
        <NuxtLink
          to="/validators"
          class="sidebar-link"
          active-class="active"
          @click="emit('close')"
        >
          <span class="link-icon">⛓</span> Validators
        </NuxtLink>
        <NuxtLink
          to="/stats"
          class="sidebar-link"
          active-class="active"
          @click="emit('close')"
        >
          <span class="link-icon">💰</span> Tokenomics
        </NuxtLink>
      </nav>

      <!-- Footer area in sidebar -->
      <div class="border-t border-border/60 pt-4 text-center">
        <a
          href="https://github.com/ARUNA-Network/aruna-core"
          target="_blank"
          rel="noopener"
          class="text-xs font-semibold text-text-muted hover:text-brand-primary transition-colors"
        >
          GitHub Repository
        </a>
      </div>
    </aside>
  </div>
</template>

<style scoped>
.sidebar {
  background: hsl(225, 28%, 8%);
}

.sidebar-link {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: var(--r-md);
  color: var(--text-secondary);
  font-weight: 500;
  font-size: 14px;
  transition: background var(--t-fast), color var(--t-fast);
}

.sidebar-link:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.sidebar-link.active {
  background: hsla(258, 85%, 65%, 0.15);
  color: var(--brand-primary);
  font-weight: 600;
  border: 1px solid hsla(258, 85%, 65%, 0.25);
}

.link-icon {
  font-size: 16px;
}
</style>
