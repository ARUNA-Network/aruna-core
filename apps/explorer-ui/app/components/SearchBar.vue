<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from '#app'
import { search } from '~/services/api'

const props = defineProps<{
  placeholder?: string
  small?: boolean
}>()

const query = ref('')
const loading = ref(false)
const errorMsg = ref('')
const router = useRouter()

async function handleSearch() {
  const q = query.value.trim()
  if (!q) return

  loading.value = true
  errorMsg.value = ''

  try {
    const results = await search(q)
    if (results && results.length > 0) {
      const match = results[0]
      if (match.kind === 'block') {
        router.push(`/block/${encodeURIComponent(match.value)}`)
      } else if (match.kind === 'transaction') {
        router.push(`/transaction/${encodeURIComponent(match.value)}`)
      } else if (match.kind === 'address') {
        router.push(`/address/${encodeURIComponent(match.value)}`)
      }
    } else {
      errorMsg.value = 'No matches found for search query.'
    }
  } catch (err) {
    if (/^\d+$/.test(q)) {
      router.push(`/block/${q}`)
    } else {
      errorMsg.value = (err as Error).message || 'Search failed.'
    }
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="search-container">
    <form :class="[small ? 'nav-search' : 'hero-search']" @submit.prevent="handleSearch" role="search">
      <input
        v-model="query"
        type="search"
        :placeholder="placeholder || 'Search by block height, transaction hash, or address...'"
        autocomplete="off"
        required
        aria-required="true"
        :disabled="loading"
      />
      <button type="submit" aria-label="Search" :disabled="loading">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
        <span v-if="!small">Search</span>
      </button>
    </form>
    <div v-if="errorMsg" class="search-error-toast">{{ errorMsg }}</div>
  </div>
</template>

<style scoped>
.search-container {
  position: relative;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.search-error-toast {
  position: absolute;
  top: 105%;
  background: var(--error);
  color: #fff;
  font-size: 13px;
  padding: 6px 12px;
  border-radius: var(--r-sm);
  z-index: 10;
  box-shadow: var(--shadow-card);
  animation: fadeIn 200ms ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
