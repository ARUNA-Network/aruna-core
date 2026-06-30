<script setup lang="ts">
import { watch } from 'vue'
import { useRoute, useAsyncData, useSeoMeta } from '#app'
import { storeToRefs } from 'pinia'
import { useBlockStore } from '~/stores/block'
import { numFmt, shortHash, microAruToAru } from '~/utils/format'
import BlockCard from '~/components/BlockCard.vue'
import Badge from '~/components/ui/badge/Badge.vue'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'

const route = useRoute()
const blockStore = useBlockStore()
const { currentBlock: block, loading, error: errorMsg } = storeToRefs(blockStore)

const height = route.params.height as string

// Fetch initial data on the server during SSR, bound to the block height parameter
await useAsyncData(['block-details', height], async () => {
  await blockStore.fetchBlockDetails(height)
  return true
})

useSeoMeta({
  title: () => `Block #${block.value?.height || height} | ARUNA Network Explorer`,
  ogTitle: () => `Block #${block.value?.height || height} | ARUNA Network Explorer`,
  description: () => `View details, transactions lists, and consensus parameters for block #${block.value?.height || height} on the ARUNA Network.`
})

watch(() => route.params.height, async (newHeight) => {
  await blockStore.fetchBlockDetails(newHeight as string)
})
</script>

<template>
  <main class="container page-spacing animate-fade">
    <!-- Breadcrumb -->
    <div class="breadcrumb">
      <NuxtLink to="/">Home</NuxtLink>
      <span class="divider">/</span>
      <NuxtLink to="/blocks">Blocks</NuxtLink>
      <span class="divider">/</span>
      <span v-if="block" class="active">Block #{{ numFmt(block.height) }}</span>
      <span v-else class="active">Block</span>
    </div>

    <!-- Header info -->
    <div class="page-header" v-if="block">
      <h1 class="page-title">Block <span class="text-glow">#{{ numFmt(block.height) }}</span></h1>
      <p class="page-subtitle mono">{{ block.hash }}</p>
    </div>

    <div v-if="loading && !block" class="flex flex-col gap-4 py-8">
      <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
      <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
      <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
    </div>
    <div v-else-if="errorMsg" class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load block details</p>
      <span class="error-msg">{{ errorMsg }}</span>
    </div>
    <div v-else-if="block" class="flex flex-col gap-6">
      <!-- ── Modular Card ── -->
      <BlockCard :block="block" />

      <!-- ── Transactions List Panel ── -->
      <Card>
        <CardHeader>
          <CardTitle>
            <span class="panel-icon">⚡</span> Block Transactions ({{ block.tx_count }})
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div v-if="!block.transactions || block.transactions.length === 0" class="empty-state">
            No transactions in this block.
          </div>
          <div v-else class="flex flex-col gap-2" role="list">
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
