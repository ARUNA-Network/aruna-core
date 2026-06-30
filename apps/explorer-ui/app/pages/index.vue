<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useNetworkStore } from '~/stores/network'
import { useBlockStore } from '~/stores/block'
import { numFmt, timeAgo, timestamp, shortHash, microAruToAru } from '~/utils/format'
import SearchBar from '~/components/common/SearchBar.vue'
import DifficultyChart from '~/components/charts/DifficultyChart.vue'

// UI Primitives
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import Badge from '~/components/ui/badge/Badge.vue'

const networkStore = useNetworkStore()
const blockStore = useBlockStore()

const { stats } = storeToRefs(networkStore)
const { latestBlock, blocksPage: recentBlocks, latestBlockError, latestTxsError, loading } = storeToRefs(blockStore)

let timer: NodeJS.Timeout | null = null

async function loadData() {
  await Promise.all([
    networkStore.fetchNetworkData(),
    blockStore.fetchLatestBlock(),
    blockStore.fetchBlocksPage(10, 0)
  ])
}

onMounted(() => {
  loadData()
  timer = setInterval(loadData, 12000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div>
    <!-- Hero Search Area -->
    <section class="hero" aria-label="Search blockchain">
      <div class="container hero-inner">
        <div class="hero-title">
          <h1>ARUNA Block Explorer</h1>
          <p class="hero-sub">Dari Rakyat. Oleh Rakyat. Untuk Rakyat. · Mine Anywhere. Owned By Everyone.</p>
        </div>
        <SearchBar />
      </div>
    </section>

    <!-- Main Grid Content -->
    <main class="container">
      <div class="dashboard-grid">
        <!-- Panel 1: Latest Block -->
        <Card id="latest-block-panel" aria-label="Latest Block Details">
          <CardHeader>
            <CardTitle>
              <span class="panel-icon">📦</span> Latest Block
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div v-if="loading && !latestBlock" class="skeleton-wrapper">
              <div class="skeleton-row"></div>
              <div class="skeleton-row"></div>
              <div class="skeleton-row"></div>
              <div class="skeleton-row"></div>
            </div>
            <div v-else-if="latestBlockError" class="error-state">
              <span class="error-icon">⚠️</span>
              <p>Failed to load data</p>
              <span class="error-msg">{{ latestBlockError }}</span>
            </div>
            <div v-else-if="latestBlock" class="detail-container">
              <div class="detail-row">
                <span class="detail-label">Height</span>
                <span class="detail-value">
                  <NuxtLink :to="`/block/${latestBlock.height}`">#{{ numFmt(latestBlock.height) }}</NuxtLink>
                </span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Hash</span>
                <span class="detail-value mono">{{ latestBlock.hash }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Timestamp</span>
                <span class="detail-value">{{ timestamp(latestBlock.timestamp) }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Difficulty</span>
                <span class="detail-value">{{ numFmt(latestBlock.difficulty) }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Nonce</span>
                <span class="detail-value">{{ numFmt(latestBlock.nonce) }}</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Panel 2: Latest Transactions -->
        <Card id="latest-txs-panel" aria-label="Latest Transactions">
          <CardHeader>
            <CardTitle>
              <span class="panel-icon">⚡</span> Latest Transactions
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div v-if="loading && !latestBlock" class="skeleton-wrapper">
              <div class="skeleton-row"></div>
              <div class="skeleton-row"></div>
              <div class="skeleton-row"></div>
            </div>
            <div v-else-if="latestTxsError" class="error-state">
              <span class="error-icon">⚠️</span>
              <p>Failed to load data</p>
              <span class="error-msg">{{ latestTxsError }}</span>
            </div>
            <div v-else-if="latestBlock">
              <div v-if="!latestBlock.transactions || latestBlock.transactions.length === 0" class="empty-state">
                No transactions in the latest block.
              </div>
              <div v-else class="list-container" role="list">
                <NuxtLink
                  v-for="tx in latestBlock.transactions.slice(0, 5)"
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
          </CardContent>
        </Card>
      </div>

      <!-- Network Status Card Grid -->
      <Card class="spacing-top" id="network-status-panel" aria-label="Network Status">
        <CardHeader>
          <CardTitle>
            <span class="panel-icon">🌐</span> Network Status
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div class="stats-grid" id="stats-grid" role="region" aria-live="polite">
            <div class="stat-card">
              <div class="stat-icon">📦</div>
              <div :class="['stat-value', { skeleton: !stats }]">
                {{ stats ? numFmt(stats.height) : '—' }}
              </div>
              <div class="stat-label">Block Height</div>
            </div>
            <div class="stat-card">
              <div class="stat-icon">⚡</div>
              <div :class="['stat-value', { skeleton: !stats }]">
                {{ stats ? numFmt(stats.total_tx_count) : '—' }}
              </div>
              <div class="stat-label">Total Transactions</div>
            </div>
            <div class="stat-card">
              <div class="stat-icon">⏱</div>
              <div :class="['stat-value', { skeleton: !stats }]">
                {{ stats ? timeAgo(stats.last_block_time) : '—' }}
              </div>
              <div class="stat-label">Last Block Time</div>
            </div>
            <div class="stat-card">
              <div class="stat-icon">👥</div>
              <div :class="['stat-value', { skeleton: !stats }]">
                {{ stats?.node ? numFmt(stats.node.peer_count) : '0' }}
              </div>
              <div class="stat-label">Connected Peers</div>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Charts Section -->
      <Card class="spacing-top" id="charts-panel" aria-label="Mining Statistics Charts">
        <CardHeader>
          <CardTitle>
            <span class="panel-icon">📈</span> Difficulty & Transaction History
          </CardTitle>
        </CardHeader>
        <CardContent class="charts-container">
          <DifficultyChart :blocks="recentBlocks" />
        </CardContent>
      </Card>
    </main>
  </div>
</template>

<style scoped>
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
