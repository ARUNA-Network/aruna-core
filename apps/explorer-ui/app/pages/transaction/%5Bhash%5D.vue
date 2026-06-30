<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute } from '#app'
import { getTransaction } from '~/services/api'
import { numFmt, shortHash, microAruToAru } from '~/utils/format'
import type { Transaction } from '~/types'

const route = useRoute()
const tx = ref<Transaction | null>(null)
const loading = ref(true)
const errorMsg = ref('')

async function loadTx() {
  loading.value = true
  errorMsg.value = ''
  const hash = route.params.hash as string
  try {
    const data = await getTransaction(hash)
    tx.value = data
  } catch (err) {
    errorMsg.value = (err as Error).message || 'Failed to load transaction details.'
  } finally {
    loading.value = false
  }
}

watch(() => route.params.hash, () => {
  loadTx()
})

onMounted(() => {
  loadTx()
})
</script>

<template>
  <main class="container page-spacing">
    <!-- Breadcrumb -->
    <div class="breadcrumb">
      <NuxtLink to="/">Home</NuxtLink>
      <span class="divider">/</span>
      <NuxtLink to="/transactions">Transactions</NuxtLink>
      <span class="divider">/</span>
      <span v-if="tx" class="active">Tx {{ shortHash(tx.hash) }}</span>
      <span v-else class="active">Transaction</span>
    </div>

    <!-- Header info card -->
    <div class="page-header" v-if="tx">
      <h1 class="page-title">Transaction Details</h1>
      <p class="page-subtitle mono">{{ tx.hash }}</p>
    </div>

    <div v-if="loading" class="skeleton-wrapper">
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
    </div>
    <div v-else-if="errorMsg" class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load transaction details</p>
      <span class="error-msg">{{ errorMsg }}</span>
    </div>
    <div v-else-if="tx" class="tx-details-panel">
      <section class="panel">
        <div class="panel-header">
          <h2 class="panel-title"><span class="panel-icon">⚡</span> Transaction Metrics</h2>
        </div>
        <div class="panel-body">
          <div class="detail-container">
            <div class="detail-row">
              <span class="detail-label">Status</span>
              <span class="detail-value">
                <span class="tag-confirmed">✓ Confirmed</span>
              </span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Hash</span>
              <span class="detail-value mono">{{ tx.hash }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Block</span>
              <span class="detail-value">
                <NuxtLink :to="`/block/${tx.block_height}`">#{{ numFmt(tx.block_height) }}</NuxtLink>
              </span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Block Hash</span>
              <span class="detail-value mono">
                <NuxtLink :to="`/block/${tx.block_hash}`">{{ shortHash(tx.block_hash) }}</NuxtLink>
              </span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Index in Block</span>
              <span class="detail-value">{{ tx.tx_index }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">From</span>
              <span class="detail-value mono">
                <NuxtLink :to="`/address/${tx.sender}`">{{ tx.sender }}</NuxtLink>
              </span>
            </div>
            <div class="detail-row">
              <span class="detail-label">To</span>
              <span class="detail-value mono">
                <NuxtLink :to="`/address/${tx.recipient}`">{{ tx.recipient }}</NuxtLink>
              </span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Amount</span>
              <span class="detail-value text-glow">{{ microAruToAru(tx.amount) }} ARU</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Transaction Fee</span>
              <span class="detail-value">{{ microAruToAru(tx.fee) }} ARU</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Nonce</span>
              <span class="detail-value">{{ numFmt(tx.nonce_val) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Gas Limit</span>
              <span class="detail-value">{{ numFmt(tx.gas_limit) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Gas Price</span>
              <span class="detail-value">{{ numFmt(tx.gas_price) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Signature Scheme</span>
              <span class="detail-value">{{ tx.sig_type === 0 ? 'Ed25519 (Consensus/Wallet)' : 'secp256k1 (EVM)' }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Has Data Payload</span>
              <span class="detail-value">{{ tx.has_data ? 'Yes' : 'No' }}</span>
            </div>
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
.tag-confirmed {
  background: hsla(142, 72%, 48%, 0.15);
  color: var(--success);
  border: 1px solid hsla(142, 72%, 48%, 0.3);
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 600;
}
</style>
