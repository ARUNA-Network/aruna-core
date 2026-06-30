<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useAsyncData, useSeoMeta } from '#app'
import { useBlockStore } from '~/stores/block'
import SearchBar from '~/components/SearchBar.vue'
import LatestTransactions from '~/components/LatestTransactions.vue'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'

const blockStore = useBlockStore()
const { latestBlock, loading, latestTxsError: errorMsg } = storeToRefs(blockStore)

// Server side data prefetch for SSR SEO bots
await useAsyncData('latest-block-txs', async () => {
  await blockStore.fetchLatestBlock()
  return true
})

useSeoMeta({
  title: 'Transactions | ARUNA Network Explorer',
  ogTitle: 'Transactions | ARUNA Network Explorer',
  description: 'Search and browse confirmed transactions, values, fees, gas limits, and signature parameters on the ARUNA Network.'
})
</script>

<template>
  <main class="container page-spacing">
    <Card>
      <CardHeader>
        <CardTitle><span class="panel-icon">⚡</span> Transactions</CardTitle>
      </CardHeader>
      <CardContent>
        <div class="search-prompt-box">
          <p class="search-prompt-label">Search for a specific transaction by hash:</p>
          <SearchBar placeholder="Enter transaction hash..." />
        </div>

        <div class="spacing-top">
          <h2 class="section-title">Transactions in Latest Block <span v-if="latestBlock" class="text-glow">#{{ latestBlock.height }}</span></h2>
          <div v-if="errorMsg" class="error-state">
            <span class="error-icon">⚠️</span>
            <p>Failed to load latest block transactions</p>
            <span class="error-msg">{{ errorMsg }}</span>
          </div>
          <div v-else>
            <LatestTransactions :transactions="latestBlock?.transactions || []" :loading="loading" />
          </div>
        </div>
      </CardContent>
    </Card>
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
</style>
