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
const { addressDetails, loading, addressError: errorMsg } = storeToRefs(txStore)

async function loadAddress() {
  const address = route.params.address as string
  await txStore.fetchAddressDetails(address, 20, 0)
}

watch(() => route.params.address, () => {
  loadAddress()
})

onMounted(() => {
  loadAddress()
})
</script>

<template>
  <main class="container page-spacing">
    <!-- Breadcrumb -->
    <div class="breadcrumb">
      <NuxtLink to="/">Home</NuxtLink>
      <span class="divider">/</span>
      <span class="active">Address</span>
    </div>

    <!-- Header info card -->
    <div class="page-header">
      <h1 class="page-title">Account Address</h1>
      <p class="page-subtitle mono">{{ route.params.address }}</p>
    </div>

    <div v-if="loading && !addressDetails" class="skeleton-wrapper">
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
    </div>
    <div v-else-if="errorMsg" class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load address details</p>
      <span class="error-msg">{{ errorMsg }}</span>
    </div>
    <div v-else-if="addressDetails" class="address-details-grid">
      <!-- ── Details Panel ── -->
      <Card>
        <CardHeader>
          <CardTitle><span class="panel-icon">💳</span> Account Metrics</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="detail-container">
            <div class="detail-row">
              <span class="detail-label">Address</span>
              <span class="detail-value mono">{{ route.params.address }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Balance</span>
              <span class="detail-value text-glow">{{ microAruToAru(addressDetails.balance) }} ARU</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Nonce</span>
              <span class="detail-value">{{ numFmt(addressDetails.nonce) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Last Updated Block</span>
              <span class="detail-value">
                <NuxtLink :to="`/block/${addressDetails.updated_at_block}`">#{{ numFmt(addressDetails.updated_at_block) }}</NuxtLink>
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- ── Transactions History List ── -->
      <Card class="spacing-top">
        <CardHeader>
          <CardTitle>
            <span class="panel-icon">🔄</span> Transaction History ({{ addressDetails.transactions?.length || 0 }})
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div v-if="!addressDetails.transactions || addressDetails.transactions.length === 0" class="empty-state">
            No transactions found for this address.
          </div>
          <div v-else class="list-container" role="list">
            <NuxtLink
              v-for="tx in addressDetails.transactions"
              :key="tx.hash"
              :to="`/transaction/${tx.hash}`"
              class="list-item tx-row"
              role="listitem"
              :aria-label="`Transaction ${tx.hash}`"
            >
              <span class="hash-short">{{ shortHash(tx.hash) }}</span>
              <span class="item-meta" @click.stop>
                <NuxtLink :to="`/address/${tx.sender}`" :class="{ 'text-glow': tx.sender === route.params.address }">
                  {{ shortHash(tx.sender) }}
                </NuxtLink>
                →
                <NuxtLink :to="`/address/${tx.recipient}`" :class="{ 'text-glow': tx.recipient === route.params.address }">
                  {{ shortHash(tx.recipient) }}
                </NuxtLink>
              </span>
              <Badge variant="default" class="amount-badge">{{ microAruToAru(tx.amount) }} ARU</Badge>
            </NuxtLink>
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
.list-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
