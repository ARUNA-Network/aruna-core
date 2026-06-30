<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getLatestBlock } from '~/services/api'
import { shortHash, microAruToAru } from '~/utils/format'
import type { Block, Transaction } from '~/types'
import SearchBar from '~/components/common/SearchBar.vue'

const latestBlock = ref<Block | null>(null)
const txs = ref<Transaction[]>([])
const loading = ref(true)
const errorMsg = ref('')

async function fetchLatestTransactions() {
  loading.value = true
  errorMsg.value = ''
  try {
    const block = await getLatestBlock()
    latestBlock.value = block
    txs.value = block.transactions || []
  } catch (err) {
    errorMsg.value = (err as Error).message || 'Failed to load latest transactions.'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchLatestTransactions()
})
</script>

<template>
  <main class="container page-spacing">
    <section class="panel">
      <div class="panel-header">
        <h1 class="panel-title"><span class="panel-icon">⚡</span> Transactions</h1>
      </div>
      <div class="panel-body">
        <div class="search-prompt-box">
          <p class="search-prompt-label">Search for a specific transaction by hash:</p>
          <SearchBar placeholder="Enter transaction hash..." />
        </div>

        <div class="spacing-top">
          <h2 class="section-title">Transactions in Latest Block <span v-if="latestBlock" class="text-glow">#{{ latestBlock.height }}</span></h2>
          <div v-if="loading" class="skeleton-wrapper">
            <div class="skeleton-row"></div>
            <div class="skeleton-row"></div>
          </div>
          <div v-else-if="errorMsg" class="error-state">
            <span class="error-icon">⚠️</span>
            <p>Failed to load latest block transactions</p>
            <span class="error-msg">{{ errorMsg }}</span>
          </div>
          <div v-else>
            <div v-if="txs.length === 0" class="empty-state">
              No transactions in the latest block.
            </div>
            <div v-else class="list-container" role="list">
              <NuxtLink
                v-for="tx in txs"
                :key="tx.hash"
                :to="`/transaction/${tx.hash}`"
                class="list-item tx-row"
                role="listitem"
                :aria-label="`Transaction ${tx.hash}`"
              >
                <span class="hash-short">{{ shortHash(tx.hash) }}</span>
                <span class="item-meta" @click.stop>
                  <NuxtLink :to="`/address/${tx.sender}`">{{ shortHash(tx.sender) }}</NuxtLink>
                  →
                  <NuxtLink :to="`/address/${tx.recipient}`">{{ shortHash(tx.recipient) }}</NuxtLink>
                </span>
                <span class="amount-badge">{{ microAruToAru(tx.amount) }} ARU</span>
              </NuxtLink>
            </div>
          </div>
        </div>
      </div>
    </section>
  </main>
</template>

<style scoped>
.search-prompt-box {
  background: var(--bg-elevated);
  padding: var(--sp-lg);
  border-radius: var(--r-md);
  border: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: var(--sp-sm);
  align-items: center;
}

.search-prompt-label {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: var(--sp-md);
  color: var(--text-primary);
}

.skeleton-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.list-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
