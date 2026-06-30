<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useRoute } from '#app'
import { storeToRefs } from 'pinia'
import { useTxStore } from '~/stores/tx'
import { numFmt, shortHash, microAruToAru } from '~/utils/format'

// UI Primitives
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import Badge from '~/components/ui/badge/Badge.vue'

const route = useRoute()
const txStore = useTxStore()
const { currentTx: tx, loading, error: errorMsg } = storeToRefs(txStore)

async function loadTx() {
  const hash = route.params.hash as string
  await txStore.fetchTransactionDetails(hash)
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

    <div v-if="loading && !tx" class="skeleton-wrapper">
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
      <Card>
        <CardHeader>
          <CardTitle><span class="panel-icon">⚡</span> Transaction Metrics</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="detail-container">
            <div class="detail-row">
              <span class="detail-label">Status</span>
              <span class="detail-value">
                <Badge variant="success">✓ Confirmed</Badge>
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
        </CardContent>
      </Card>
    </div>
  </main>
</template>

<style scoped>
.skeleton-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
</style>
