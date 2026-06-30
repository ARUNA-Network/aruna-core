<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getStatus, getLatestBlock, getBlocks } from '~/services/api'
import { numFmt, timeAgo, timestamp, shortHash, microAruToAru } from '~/utils/format'
import type { Stats, Block } from '~/types'
import SearchBar from '~/components/common/SearchBar.vue'
import DifficultyChart from '~/components/charts/DifficultyChart.vue'

const stats = ref<Stats | null>(null)
const latestBlock = ref<Block | null>(null)
const recentBlocks = ref<Block[]>([])
const loading = ref(true)
const latestBlockError = ref('')
const latestTxsError = ref('')
let timer: NodeJS.Timeout | null = null

async function loadData() {
  try {
    const statsData = await getStatus()
    stats.value = statsData
  } catch (err) {
    console.warn('Failed to load status stats:', err)
  }

  try {
    const block = await getLatestBlock()
    latestBlock.value = block
    latestBlockError.value = ''
    latestTxsError.value = ''
  } catch (err) {
    latestBlockError.value = (err as Error).message || 'Failed to load latest block.'
    latestTxsError.value = (err as Error).message || 'Failed to load latest transactions.'
  }

  try {
    const blocksList = await getBlocks(10, 0)
    recentBlocks.value = blocksList
  } catch (err) {
    console.warn('Failed to load blocks for chart:', err)
  }

  loading.value = false
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
        <section class="panel" id="latest-block-panel" aria-label="Latest Block Details">
          <div class="panel-header">
            <h2 class="panel-title">
              <span class="panel-icon">📦</span> Latest Block
            </h2>
          </div>
          <div class="panel-body">
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
          </div>
        </section>

        <!-- Panel 2: Latest Transactions -->
        <section class="panel" id="latest-txs-panel" aria-label="Latest Transactions">
          <div class="panel-header">
            <h2 class="panel-title">
              <span class="panel-icon">⚡</span> Latest Transactions
            </h2>
          </div>
          <div class="panel-body">
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
                  <span class="amount-badge">{{ microAruToAru(tx.amount) }} ARU</span>
                </NuxtLink>
              </div>
            </div>
          </div>
        </section>
      </div>

      <!-- Network Status Card Grid -->
      <section class="panel spacing-top" id="network-status-panel" aria-label="Network Status">
        <div class="panel-header">
          <h2 class="panel-title">
            <span class="panel-icon">🌐</span> Network Status
          </h2>
        </div>
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
      </section>

      <!-- Charts Section -->
      <section class="panel spacing-top" id="charts-panel" aria-label="Mining Statistics Charts">
        <div class="panel-header">
          <h2 class="panel-title">
            <span class="panel-icon">📈</span> Difficulty & Transaction History
          </h2>
        </div>
        <div class="charts-container">
          <DifficultyChart :blocks="recentBlocks" />
        </div>
      </section>
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
