<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useRoute, useRouter } from '#app'
import { getBlocks } from '~/services/api'
import { numFmt, timestamp, shortHash } from '~/utils/format'
import type { Block } from '~/types'

const limit = 20
const blocks = ref<Block[]>([])
const loading = ref(true)
const errorMsg = ref('')
const route = useRoute()
const router = useRouter()

const currentPage = ref(Number(route.query.page) || 1)

async function fetchBlocks() {
  loading.value = true
  errorMsg.value = ''
  const offset = (currentPage.value - 1) * limit
  try {
    const data = await getBlocks(limit, offset)
    blocks.value = data
  } catch (err) {
    errorMsg.value = (err as Error).message || 'Failed to load blocks list.'
  } finally {
    loading.value = false
  }
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
    <section class="panel">
      <div class="panel-header">
        <h1 class="panel-title"><span class="panel-icon">📦</span> Blocks</h1>
      </div>
      <div class="panel-body">
        <div v-if="loading" class="skeleton-wrapper">
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
        </div>
        <div v-else-if="errorMsg" class="error-state">
          <span class="error-icon">⚠️</span>
          <p>Failed to load blocks</p>
          <span class="error-msg">{{ errorMsg }}</span>
        </div>
        <div v-else>
          <div v-if="blocks.length === 0" class="empty-state">
            No blocks found.
          </div>
          <div v-else>
            <table class="grid-table">
              <thead>
                <tr>
                  <th>Height</th>
                  <th>Hash</th>
                  <th>Timestamp</th>
                  <th>Transactions</th>
                  <th>Difficulty</th>
                  <th>Nonce</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="block in blocks" :key="block.hash">
                  <td>
                    <NuxtLink :to="`/block/${block.height}`">#{{ numFmt(block.height) }}</NuxtLink>
                  </td>
                  <td class="mono">
                    <NuxtLink :to="`/block/${block.hash}`">{{ shortHash(block.hash) }}</NuxtLink>
                  </td>
                  <td>{{ timestamp(block.timestamp) }}</td>
                  <td>{{ numFmt(block.tx_count) }}</td>
                  <td>{{ numFmt(block.difficulty) }}</td>
                  <td>{{ numFmt(block.nonce) }}</td>
                </tr>
              </tbody>
            </table>

            <!-- Pagination Buttons -->
            <div class="pagination-bar">
              <button
                :disabled="currentPage <= 1"
                @click="navigateToPage(currentPage - 1)"
                class="btn-nav"
              >
                ← Previous
              </button>
              <span class="page-indicator">Page {{ currentPage }}</span>
              <button
                :disabled="blocks.length < limit"
                @click="navigateToPage(currentPage + 1)"
                class="btn-nav"
              >
                Next →
              </button>
            </div>
          </div>
        </div>
      </div>
    </section>
  </main>
</template>

<style scoped>
.skeleton-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.pagination-bar {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: var(--sp-md);
  margin-top: var(--sp-lg);
  padding-top: var(--sp-md);
  border-top: 1px solid var(--border);
}

.btn-nav {
  background: var(--bg-panel);
  border: 1px solid var(--border);
  color: var(--text-primary);
  padding: 8px 16px;
  border-radius: var(--r-sm);
  cursor: pointer;
  font-family: inherit;
  transition: border-color var(--t-fast), background var(--t-fast);
}

.btn-nav:hover:not(:disabled) {
  border-color: var(--brand-primary);
  background: var(--bg-hover);
}

.btn-nav:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.page-indicator {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}
</style>
