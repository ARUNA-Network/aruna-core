<script setup lang="ts">
import { clearError } from '#app'

const props = defineProps({
  error: Object
})

function handleError() {
  clearError({ redirect: '/' })
}
</script>

<template>
  <main class="container page-spacing">
    <div class="error-state-box">
      <span class="error-icon">⚠️</span>
      <h1 class="error-title">System Error</h1>
      <p class="error-message">{{ error?.statusCode === 404 ? 'The requested page does not exist.' : error?.message || 'An unexpected error occurred.' }}</p>
      <button @click="handleError" class="btn-home">Return Home</button>
    </div>
  </main>
</template>

<style scoped>
.error-state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  gap: var(--sp-md);
  text-align: center;
}

.error-icon {
  font-size: 48px;
}

.error-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
}

.error-message {
  font-size: 15px;
  color: var(--text-secondary);
  max-width: 480px;
}

.btn-home {
  background: var(--brand-primary);
  color: #fff;
  border: none;
  padding: 10px 24px;
  border-radius: var(--r-sm);
  cursor: pointer;
  font-family: inherit;
  font-weight: 600;
  transition: opacity var(--t-fast);
  box-shadow: var(--shadow-glow);
}

.btn-home:hover {
  opacity: 0.9;
}
</style>
