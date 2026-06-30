<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useRoute } from '#app'
import { storeToRefs } from 'pinia'
import { useTxStore } from '~/stores/tx'
import { shortHash } from '~/utils/format'
import TransactionCard from '~/components/TransactionCard.vue'

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
  <main class="container page-spacing animate-fade">
    <!-- Breadcrumb -->
    <div class="breadcrumb">
      <NuxtLink to="/">Home</NuxtLink>
      <span class="divider">/</span>
      <NuxtLink to="/transactions">Transactions</NuxtLink>
      <span class="divider">/</span>
      <span v-if="tx" class="active">Tx {{ shortHash(tx.hash) }}</span>
      <span v-else class="active">Transaction</span>
    </div>

    <!-- Header info -->
    <div class="page-header" v-if="tx">
      <h1 class="page-title">Transaction Details</h1>
      <p class="page-subtitle mono">{{ tx.hash }}</p>
    </div>

    <div v-if="loading && !tx" class="flex flex-col gap-4 py-8">
      <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
      <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
    </div>
    <div v-else-if="errorMsg" class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load transaction details</p>
      <span class="error-msg">{{ errorMsg }}</span>
    </div>
    <div v-else-if="tx" class="tx-details-panel">
      <!-- ── Modular Card ── -->
      <TransactionCard :tx="tx" />
    </div>
  </main>
</template>

<style scoped>
.animate-fade {
  animation: fadeIn 200ms ease;
}
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
