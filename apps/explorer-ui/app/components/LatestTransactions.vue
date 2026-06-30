<script setup lang="ts">
import type { Transaction } from '~/types'
import { shortHash, microAruToAru } from '~/utils/format'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import Badge from '~/components/ui/badge/Badge.vue'

defineProps<{
  transactions: Transaction[]
  loading: boolean
}>()
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle><span class="panel-icon">⚡</span> Recent Transactions</CardTitle>
    </CardHeader>
    <CardContent>
      <div v-if="loading && transactions.length === 0" class="flex flex-col gap-3">
        <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
        <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
      </div>
      <div v-else>
        <div v-if="transactions.length === 0" class="text-center py-6 text-text-muted">
          No transactions found.
        </div>
        <div v-else class="flex flex-col gap-2" role="list">
          <NuxtLink
            v-for="tx in transactions"
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
</template>

<style scoped>
.list-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
