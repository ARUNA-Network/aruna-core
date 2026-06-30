<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useNetworkStore } from '~/stores/network'
import { numFmt } from '~/utils/format'

// UI Primitives
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import Button from '~/components/ui/button/Button.vue'
import Badge from '~/components/ui/badge/Badge.vue'
import Table from '~/components/ui/table/Table.vue'
import TableHeader from '~/components/ui/table/TableHeader.vue'
import TableBody from '~/components/ui/table/TableBody.vue'
import TableRow from '~/components/ui/table/TableRow.vue'
import TableHead from '~/components/ui/table/TableHead.vue'
import TableCell from '~/components/ui/table/TableCell.vue'

const networkStore = useNetworkStore()
const { stats, network, loading, error: errorMsg } = storeToRefs(networkStore)

const activeTab = ref<'status' | 'peers'>('status')

onMounted(() => {
  networkStore.fetchNetworkData()
})
</script>

<template>
  <main class="container page-spacing">
    <!-- Header -->
    <div class="page-header">
      <h1 class="page-title">Network Diagnostics</h1>
      <p class="page-subtitle">ARUNA Node P2P Connectivity & Consensus State</p>
    </div>

    <!-- Tabs selector -->
    <div class="tabs-container">
      <Button
        :variant="activeTab === 'status' ? 'default' : 'secondary'"
        @click="activeTab = 'status'"
        size="sm"
      >
        🌐 Core Metrics
      </Button>
      <Button
        :variant="activeTab === 'peers' ? 'default' : 'secondary'"
        @click="activeTab = 'peers'"
        size="sm"
      >
        👥 Connected Peers ({{ network?.peers?.length || 0 }})
      </Button>
    </div>

    <div v-if="loading && !stats" class="skeleton-wrapper">
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
      <div class="skeleton-row"></div>
    </div>
    <div v-else-if="errorMsg" class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load network diagnostics</p>
      <span class="error-msg">{{ errorMsg }}</span>
    </div>
    <div v-else>
      <!-- ── Tab 1: Core Metrics ── -->
      <div v-if="activeTab === 'status' && stats" class="network-status-grid animate-fade">
        <Card>
          <CardHeader>
            <CardTitle><span class="panel-icon">🔧</span> System Configuration</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="detail-container">
              <div class="detail-row">
                <span class="detail-label">Network Name</span>
                <span class="detail-value">{{ stats.node ? stats.node.network : 'Sumatera Testnet' }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Chain ID</span>
                <span class="detail-value">{{ stats.node ? stats.node.chain_id : '7777' }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Synchronization Status</span>
                <span class="detail-value">
                  <Badge v-if="stats.node?.synced" variant="success">✓ Synced</Badge>
                  <Badge v-else variant="destructive">Standalone Node</Badge>
                </span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Uptime</span>
                <span class="detail-value">
                  {{ stats.node ? `${numFmt(Math.floor(stats.node.uptime_seconds / 3600))} hours` : '—' }}
                </span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Node Version</span>
                <span class="detail-value">{{ stats.node ? stats.node.version : '0.1.0' }}</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card class="spacing-top-mobile">
          <CardHeader>
            <CardTitle><span class="panel-icon">⛓</span> Consensus Height</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="detail-container">
              <div class="detail-row">
                <span class="detail-label">Best Block Height</span>
                <span class="detail-value">
                  <NuxtLink :to="`/block/${stats.height}`">#{{ numFmt(stats.height) }}</NuxtLink>
                </span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Best Block Hash</span>
                <span class="detail-value mono text-xs leading-5">
                  <NuxtLink :to="`/block/${stats.best_hash}`">{{ stats.best_hash }}</NuxtLink>
                </span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Total Transactions</span>
                <span class="detail-value">{{ numFmt(stats.total_tx_count) }}</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- ── Tab 2: Connected Peers List ── -->
      <div v-if="activeTab === 'peers' && network" class="peers-list-wrapper animate-fade">
        <Card>
          <CardHeader>
            <CardTitle><span class="panel-icon">👥</span> Peers List</CardTitle>
          </CardHeader>
          <CardContent>
            <div v-if="!network.peers || network.peers.length === 0" class="empty-state">
              No active peers connected. Nodes operate in standalone genesis mode.
            </div>
            <div v-else>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Index</TableHead>
                    <TableHead>Peer Address (P2P)</TableHead>
                    <TableHead>Capabilities</TableHead>
                    <TableHead>Status</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow v-for="(peer, idx) in network.peers" :key="peer">
                    <TableCell class="mono">#{{ idx + 1 }}</TableCell>
                    <TableCell class="mono">{{ peer }}</TableCell>
                    <TableCell>Full Node</TableCell>
                    <TableCell><Badge variant="success">Active</Badge></TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  </main>
</template>

<style scoped>
.tabs-container {
  display: flex;
  gap: var(--sp-md);
  margin-bottom: var(--sp-lg);
  border-bottom: 1px solid var(--border);
  padding-bottom: var(--sp-md);
}

.network-status-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sp-lg);
}

@media (max-width: 768px) {
  .network-status-grid {
    grid-template-columns: 1fr;
  }
  .spacing-top-mobile {
    margin-top: var(--sp-md);
  }
}

.animate-fade {
  animation: fadeIn 220ms ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
