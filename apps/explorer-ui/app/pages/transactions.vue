<script setup lang="ts">
import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useBlockStore } from '~/stores/block'
import { shortHash, microAruToAru } from '~/utils/format'
import SearchBar from '~/components/common/SearchBar.vue'

// UI Primitives
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import Badge from '~/components/ui/badge/Badge.vue'

const blockStore = useBlockStore()
const { latestBlock, loading, latestTxsError: errorMsg } = storeToRefs(blockStore)

onMounted(() => {
  blockStore.fetchLatestBlock()
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
          <div v-if="loading && !latestBlock" class="skeleton-wrapper">
            <div class="skeleton-row"></div>
            <div class="skeleton-row"></div>
          </div>
          <div v-else-if="errorMsg" class="error-state">
            <span class="error-icon">⚠️</span>
            <p>Failed to load latest block transactions</p>
            <span class="error-msg">{{ errorMsg }}</span>
          </div>
          <div v-else-if="latestBlock">
            <div v-if="!latestBlock.transactions || latestBlock.transactions.length === 0" class="empty-state">
              No transactions in the latest block.
            </div>
            <div v-else class="list-container" role="list">
              <NuxtLink
                v-for="tx in latestBlock.transactions"
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
                <Badge variant="default" class="amount-badge">{{ microAruToAru(tx.amount) }} ARU</Badge>
              </NuxtLink>
            </div>
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
