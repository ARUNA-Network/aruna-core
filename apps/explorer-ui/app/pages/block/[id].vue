<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute } from '#app'
import { getBlock } from '~/services/api'
import { numFmt, timestamp, shortHash, microAruToAru } from '~/utils/format'
import type { Block } from '~/types'

const route = useRoute()
const block = ref<Block | null>(null)
const loading = ref(true)
const errorMsg = ref('')

async function loadBlock() {
  loading.value = true
  errorMsg.value = ''
  const id = route.params.id as string
  try {
    const data = await getBlock(id)
    block.value = data
  } catch (err) {
    errorMsg.value = (err as Error).message || 'Failed to load block details.'
  } finally {
    loading.value = false
  }
}

watch(() => route.params.id, () => {
  loadBlock()
})

onMounted(() => {
  loadBlock()
})
</script>

<template>
  <main class="container page-spacing">
    <!-- Breadcrumb -->
    <div class="breadcrumb">
      <NuxtLink to="/">Home</NuxtLink>
      <span class="divider">/</span>
      <NuxtLink to="/blocks">Blocks</NuxtLink>
      <span class="divider">/</span>
      <span v-if="block" class="active">Block #{{ numFmt(block.height) }}</span>
      <span v-else class="active">Block</span>
    </div>

    <!-- Header info card -->
    <div class="page-header" v-if="block">
      <h1 class="page-title">Block <span class="text-glow">#{{ numFmt(block.height) }}</span></h1>
      <p class="page-subtitle mono">{{ block.hash }}</p>
    </div>

    <div v-if="loading" class="skeleton-wrapper">
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
    </div>
    <div v-else-if="errorMsg" class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load block details</p>
      <span class="error-msg">{{ errorMsg }}</span>
    </div>
    <div v-else-if="block" class="block-details-grid">
      <!-- ── Details Panel ── -->
      <section class="panel">
        <div class="panel-header">
          <h2 class="panel-title"><span class="panel-icon">📝</span> Block Details</h2>
        </div>
        <div class="panel-body">
          <div class="detail-container">
            <div class="detail-row">
              <span class="detail-label">Height</span>
              <span class="detail-value">#{{ numFmt(block.height) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Hash</span>
              <span class="detail-value mono">{{ block.hash }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Previous Block</span>
              <span class="detail-value mono">
                <NuxtLink :to="`/block/${block.prev_hash}`">{{ block.prev_hash }}</NuxtLink>
              </span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Merkle Root</span>
              <span class="detail-value mono">{{ block.merkle_root }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">State Root</span>
              <span class="detail-value mono">{{ block.state_root }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Timestamp</span>
              <span class="detail-value">{{ timestamp(block.timestamp) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Difficulty</span>
              <span class="detail-value">{{ numFmt(block.difficulty) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Nonce</span>
              <span class="detail-value">{{ numFmt(block.nonce) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Version</span>
              <span class="detail-value">{{ block.version }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Transactions</span>
              <span class="detail-value">{{ block.tx_count }}</span>
            </div>
          </div>
        </div>
      </section>

      <!-- ── Transactions List Panel ── -->
      <section class="panel spacing-top">
        <div class="panel-header">
          <h2 class="panel-title">
            <span class="panel-icon">⚡</span> Block Transactions ({{ block.tx_count }})
          </h2>
        </div>
        <div class="panel-body">
          <div v-if="!block.transactions || block.transactions.length === 0" class="empty-state">
            No transactions in this block.
          </div>
          <div v-else class="list-container" role="list">
            <NuxtLink
              v-for="tx in block.transactions"
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
      </section>
    </div>
  </main>
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
