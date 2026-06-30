<script setup lang="ts">
import type { Transaction } from '~/types'
import { numFmt, shortHash, microAruToAru } from '~/utils/format'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import Badge from '~/components/ui/badge/Badge.vue'

defineProps<{
  tx: Transaction
}>()
</script>

<template>
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
          <span class="detail-value mono text-glow text-xs leading-5">{{ tx.hash }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Block</span>
          <span class="detail-value">
            <NuxtLink :to="`/block/${tx.block_height}`">#{{ numFmt(tx.block_height) }}</NuxtLink>
          </span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Block Hash</span>
          <span class="detail-value mono text-xs leading-5">
            <NuxtLink :to="`/block/${tx.block_hash}`">{{ shortHash(tx.block_hash) }}</NuxtLink>
          </span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Index in Block</span>
          <span class="detail-value">{{ tx.tx_index }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">From</span>
          <span class="detail-value mono text-xs leading-5">
            <NuxtLink :to="`/address/${tx.sender}`">{{ tx.sender }}</NuxtLink>
          </span>
        </div>
        <div class="detail-row">
          <span class="detail-label">To</span>
          <span class="detail-value mono text-xs leading-5">
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
</template>
