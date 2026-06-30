<script setup lang="ts">
import { watch } from 'vue'
import { useRoute, useAsyncData, useSeoMeta } from '#app'
import { storeToRefs } from 'pinia'
import { useTxStore } from '~/stores/tx'
import { numFmt, shortHash, microAruToAru } from '~/utils/format'
import AddressCard from '~/components/AddressCard.vue'
import Badge from '~/components/ui/badge/Badge.vue'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'

const route = useRoute()
const txStore = useTxStore()
const { addressDetails, loading, addressError: errorMsg } = storeToRefs(txStore)

const address = route.params.address as string

// Fetch initial data on the server during SSR, bound to the address parameter
await useAsyncData(['address-details', address], async () => {
  await txStore.fetchAddressDetails(address, 20, 0)
  return true
})

useSeoMeta({
  title: () => `Address ${shortHash(address)} | ARUNA Network Explorer`,
  ogTitle: () => `Address ${shortHash(address)} | ARUNA Network Explorer`,
  description: () => `View address parameters, balance, nonce, and transaction logs for address ${address} on the ARUNA Network.`
})

watch(() => route.params.address, async (newAddress) => {
  await txStore.fetchAddressDetails(newAddress as string, 20, 0)
})
</script>

<template>
  <main class="container page-spacing animate-fade">
    <!-- Breadcrumb -->
    <div class="breadcrumb">
      <NuxtLink to="/">Home</NuxtLink>
      <span class="divider">/</span>
      <span class="active">Address</span>
    </div>

    <!-- Header info -->
    <div class="page-header">
      <h1 class="page-title">Account Address</h1>
      <p class="page-subtitle mono">{{ route.params.address }}</p>
    </div>

    <div v-if="loading && !addressDetails" class="flex flex-col gap-4 py-8">
      <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
      <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
    </div>
    <div v-else-if="errorMsg" class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load address details</p>
      <span class="error-msg">{{ errorMsg }}</span>
    </div>
    <div v-else-if="addressDetails" class="flex flex-col gap-6">
      <!-- ── Modular Card ── -->
      <AddressCard :address="route.params.address as string" :address-data="addressDetails" />

      <!-- ── Transactions History List ── -->
      <Card>
        <CardHeader>
          <CardTitle>
            <span class="panel-icon">🔄</span> Transaction History ({{ addressDetails.transactions?.length || 0 }})
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div v-if="!addressDetails.transactions || addressDetails.transactions.length === 0" class="empty-state">
            No transactions found for this address.
          </div>
          <div v-else class="flex flex-col gap-2" role="list">
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
.animate-fade {
  animation: fadeIn 200ms ease;
}
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
