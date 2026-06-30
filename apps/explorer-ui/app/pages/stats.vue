<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useAsyncData, useSeoMeta } from '#app'
import { useNetworkStore } from '~/stores/network'
import { numFmt } from '~/utils/format'

// UI Primitives
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'

const networkStore = useNetworkStore()
const { stats, loading, error: errorMsg } = storeToRefs(networkStore)

const maxSupply = 1000000000
const premine = 15000000
const blockReward = 25

function circulatingSupply(height: number) {
  return premine + (height * blockReward)
}

// Fetch initial data on the server during SSR
await useAsyncData('network-stats', async () => {
  await networkStore.fetchNetworkData()
  return true
})

useSeoMeta({
  title: 'Economics & Supply | ARUNA Network Explorer',
  ogTitle: 'Economics & Supply | ARUNA Network Explorer',
  description: 'View circulating supply details, premine allocations, halving schedules, and block reward split metrics of the ARUNA token.'
})
</script>

<template>
  <main class="container page-spacing">
    <!-- Supply Overview -->
    <Card>
      <CardHeader>
        <CardTitle><span class="panel-icon">💰</span> Circulating Supply</CardTitle>
      </CardHeader>
      <CardContent>
        <div v-if="loading && !stats" class="skeleton-wrapper">
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
        </div>
        <div v-else-if="errorMsg" class="error-state">
          <span class="error-icon">⚠️</span>
          <p>Failed to load supply statistics</p>
          <span class="error-msg">{{ errorMsg }}</span>
        </div>
        <div v-else-if="stats" class="detail-container">
          <div class="detail-row">
            <span class="detail-label">Max Token Cap Limit</span>
            <span class="detail-value text-glow">{{ numFmt(maxSupply) }} ARU</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Current Block Reward</span>
            <span class="detail-value">{{ blockReward }} ARU (Era 1: Years 0–4)</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Halving Interval Era</span>
            <span class="detail-value">Every 4 Years (4,204,800 blocks)</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Genesis Premine Allocation</span>
            <span class="detail-value">1.5% ({{ numFmt(premine) }} ARU for testnet rewards & bootstrap)</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Founder Allocations Vesting</span>
            <span class="detail-value">1.5% (Vesting: 48 Months Linear, monthly lock)</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Current Circulating Supply</span>
            <span class="detail-value text-glow font-bold">{{ numFmt(circulatingSupply(stats.height)) }} ARU</span>
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- Block Reward Distribution Model -->
    <Card class="spacing-top">
      <CardHeader>
        <CardTitle><span class="panel-icon">📊</span> Block Reward Distribution Model</CardTitle>
      </CardHeader>
      <CardContent>
        <div class="detail-container">
          <div class="detail-row">
            <span class="detail-label">Proof of Work (PoW / Miners)</span>
            <span class="detail-value">70% (17.5 ARU)</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Proof of Stake (PoS / Validators)</span>
            <span class="detail-value">25% (6.25 ARU)</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Network Treasury Allocation</span>
            <span class="detail-value">5% (1.25 ARU under governance contracts control)</span>
          </div>
        </div>
      </CardContent>
    </Card>
  </main>
</template>

<style scoped>
.skeleton-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.font-bold {
  font-weight: 700;
}
</style>
