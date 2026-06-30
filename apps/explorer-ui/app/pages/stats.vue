<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getStatus } from '~/services/api'
import { numFmt } from '~/utils/format'

const height = ref(0)
const loading = ref(true)
const errorMsg = ref('')

const maxSupply = 1000000000
const premine = 15000000
const blockReward = 25

async function fetchStats() {
  loading.value = true
  errorMsg.value = ''
  try {
    const data = await getStatus()
    height.value = data.height
  } catch (err) {
    errorMsg.value = (err as Error).message || 'Failed to load stats.'
  } finally {
    loading.value = false
  }
}

const circulatingSupply = () => {
  return premine + (height.value * blockReward)
}

onMounted(() => {
  fetchStats()
})
</script>

<template>
  <main class="container page-spacing">
    <!-- Supply Overview -->
    <section class="panel">
      <div class="panel-header">
        <h1 class="panel-title"><span class="panel-icon">💰</span> Circulating Supply</h1>
      </div>
      <div class="panel-body">
        <div v-if="loading" class="skeleton-wrapper">
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
        </div>
        <div v-else-if="errorMsg" class="error-state">
          <span class="error-icon">⚠️</span>
          <p>Failed to load supply statistics</p>
          <span class="error-msg">{{ errorMsg }}</span>
        </div>
        <div v-else class="detail-container">
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
            <span class="detail-value text-glow font-bold">{{ numFmt(circulatingSupply()) }} ARU</span>
          </div>
        </div>
      </div>
    </section>

    <!-- Block Reward Distribution Model -->
    <section class="panel spacing-top">
      <div class="panel-header">
        <h2 class="panel-title"><span class="panel-icon">📊</span> Block Reward Distribution Model</h2>
      </div>
      <div class="panel-body">
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
      </div>
    </section>
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
