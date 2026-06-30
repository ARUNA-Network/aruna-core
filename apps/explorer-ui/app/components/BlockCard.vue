<script setup lang="ts">
import type { Block } from '~/types'
import { numFmt, timestamp, shortHash } from '~/utils/format'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'

defineProps<{
  block: Block
}>()
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle><span class="panel-icon">📝</span> Block Details</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="detail-container">
        <div class="detail-row">
          <span class="detail-label">Height</span>
          <span class="detail-value">#{{ numFmt(block.height) }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Hash</span>
          <span class="detail-value mono text-glow text-xs leading-5">{{ block.hash }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Previous Block</span>
          <span class="detail-value mono text-xs leading-5">
            <NuxtLink :to="`/block/${block.prev_hash}`">{{ block.prev_hash }}</NuxtLink>
          </span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Merkle Root</span>
          <span class="detail-value mono text-xs leading-5">{{ block.merkle_root }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">State Root</span>
          <span class="detail-value mono text-xs leading-5">{{ block.state_root }}</span>
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
    </CardContent>
  </Card>
</template>
