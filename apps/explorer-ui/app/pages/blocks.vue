<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useRoute, useRouter } from '#app'
import { storeToRefs } from 'pinia'
import { useBlockStore } from '~/stores/block'

// UI Primitives
import LatestBlocks from '~/components/LatestBlocks.vue'
import Button from '~/components/ui/button/Button.vue'

const limit = 20
const blockStore = useBlockStore()
const { blocksPage: blocks, loading, error: errorMsg } = storeToRefs(blockStore)

const route = useRoute()
const router = useRouter()
const currentPage = ref(Number(route.query.page) || 1)

async function fetchBlocks() {
  const offset = (currentPage.value - 1) * limit
  await blockStore.fetchBlocksPage(limit, offset)
}

watch(() => route.query.page, (newPage) => {
  currentPage.value = Number(newPage) || 1
  fetchBlocks()
})

onMounted(() => {
  fetchBlocks()
})

function navigateToPage(page: number) {
  router.push({ query: { page } })
}
</script>

<template>
  <main class="container page-spacing">
    <div v-if="errorMsg" class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load blocks</p>
      <span class="error-msg">{{ errorMsg }}</span>
    </div>
    <div v-else>
      <LatestBlocks :blocks="blocks" :loading="loading" />

      <!-- Pagination Bar -->
      <div class="pagination-bar" v-if="blocks.length > 0">
        <Button
          :disabled="currentPage <= 1"
          @click="navigateToPage(currentPage - 1)"
          variant="secondary"
        >
          ← Previous
        </Button>
        <span class="page-indicator">Page {{ currentPage }}</span>
        <Button
          :disabled="blocks.length < limit"
          @click="navigateToPage(currentPage + 1)"
          variant="secondary"
        >
          Next →
        </Button>
      </div>
    </div>
  </main>
</template>

<style scoped>
.pagination-bar {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: var(--sp-md);
  margin-top: var(--sp-lg);
  padding-top: var(--sp-md);
  border-top: 1px solid var(--border);
}

.page-indicator {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}
</style>
