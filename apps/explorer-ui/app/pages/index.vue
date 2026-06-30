<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useNetworkStore } from '~/stores/network'
import { useBlockStore } from '~/stores/block'
import SearchBar from '~/components/SearchBar.vue'
import DifficultyChart from '~/components/charts/DifficultyChart.vue'
import NetworkCard from '~/components/NetworkCard.vue'
import LatestBlocks from '~/components/LatestBlocks.vue'
import LatestTransactions from '~/components/LatestTransactions.vue'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'

const networkStore = useNetworkStore()
const blockStore = useBlockStore()

const { stats } = storeToRefs(networkStore)
const { latestBlock, blocksPage: recentBlocks, loading } = storeToRefs(blockStore)

let timer: NodeJS.Timeout | null = null

async function loadData() {
  await Promise.all([
    networkStore.fetchNetworkData(),
    blockStore.fetchLatestBlock(),
    blockStore.fetchBlocksPage(10, 0)
  ])
}

onMounted(() => {
  loadData()
  timer = setInterval(loadData, 12000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div>
    <!-- Hero Search Area -->
    <section class="hero animate-fade" aria-label="Search blockchain">
      <div class="container hero-inner">
        <div class="hero-title">
          <h1>ARUNA Block Explorer</h1>
          <p class="hero-sub">Dari Rakyat. Oleh Rakyat. Untuk Rakyat. · Mine Anywhere. Owned By Everyone.</p>
        </div>
        <SearchBar />
      </div>
    </section>

    <!-- Main Grid Content -->
    <main class="container py-6 flex flex-col gap-6">
      <!-- Network Grid Status -->
      <NetworkCard :stats="stats" />

      <!-- Latest Info Grid -->
      <div class="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <!-- Recent Blocks Card List -->
        <LatestBlocks :blocks="recentBlocks.slice(0, 5)" :loading="loading" />

        <!-- Recent Transactions Card List -->
        <LatestTransactions :transactions="latestBlock?.transactions || []" :loading="loading" />
      </div>

      <!-- Mining statistics chart -->
      <Card id="charts-panel" aria-label="Mining Statistics Charts">
        <CardHeader>
          <CardTitle>
            <span class="panel-icon">📈</span> Difficulty & Transaction History
          </CardTitle>
        </CardHeader>
        <CardContent class="charts-container">
          <DifficultyChart :blocks="recentBlocks" />
        </CardContent>
      </Card>
    </main>
  </div>
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
