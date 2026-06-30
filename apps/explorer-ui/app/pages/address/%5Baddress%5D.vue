<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute } from '#app'
import { getAddress } from '~/services/api'
import { numFmt, shortHash, microAruToAru } from '~/utils/format'
import type { AddressData } from '~/types'

const route = useRoute()
const addressData = ref<AddressData | null>(null)
const loading = ref(true)
const errorMsg = ref('')

async function loadAddress() {
  loading.value = true
  errorMsg.value = ''
  const address = route.params.address as string
  try {
    const data = await getAddress(address, 20, 0)
    addressData.value = data
  } catch (err) {
    errorMsg.value = (err as Error).message || 'Failed to load address details.'
  } finally {
    loading.value = false
  }
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

    <div v-if="loading" class="skeleton-wrapper">
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
    </div>
    <div v-else-if="errorMsg" class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load address details</p>
      <span class="error-msg">{{ errorMsg }}</span>
    </div>
    <div v-else-if="addressData" class="address-details-grid">
      <!-- ── Details Panel ── -->
      <section class="panel">
        <div class="panel-header">
          <h2 class="panel-title"><span class="panel-icon">💳</span> Account Metrics</h2>
        </div>
        <div class="panel-body">
          <div class="detail-container">
            <div class="detail-row">
              <span class="detail-label">Address</span>
              <span class="detail-value mono">{{ route.params.address }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Balance</span>
              <span class="detail-value text-glow">{{ microAruToAru(addressData.balance) }} ARU</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Nonce</span>
              <span class="detail-value">{{ numFmt(addressData.nonce) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Last Updated Block</span>
              <span class="detail-value">
                <NuxtLink :to="`/block/${addressData.updated_at_block}`">#{{ numFmt(addressData.updated_at_block) }}</NuxtLink>
              </span>
            </div>
          </div>
        </div>
      </section>

      <!-- ── Transactions History List ── -->
      <section class="panel spacing-top">
        <div class="panel-header">
          <h2 class="panel-title">
            <span class="panel-icon">🔄</span> Transaction History ({{ addressData.transactions?.length || 0 }})
          </h2>
        </div>
        <div class="panel-body">
          <div v-if="!addressData.transactions || addressData.transactions.length === 0" class="empty-state">
            No transactions found for this address.
          </div>
          <div v-else class="list-container" role="list">
            <NuxtLink
              v-for="tx in addressData.transactions"
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
