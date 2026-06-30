<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getStatus, getNetwork } from '~/services/api'
import { numFmt } from '~/utils/format'
import type { Stats, NetworkData } from '~/types'

const stats = ref<Stats | null>(null)
const network = ref<NetworkData | null>(null)
const loading = ref(true)
const errorMsg = ref('')

const activeTab = ref<'status' | 'peers'>('status')

async function loadNetworkData() {
  loading.value = true
  errorMsg.value = ''
  try {
    const [statsData, networkData] = await Promise.all([
      getStatus(),
      getNetwork()
    ])
    stats.value = statsData
    network.value = networkData
  } catch (err) {
    errorMsg.value = (err as Error).message || 'Failed to load network status.'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadNetworkData()
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
      <button
        :class="['tab-btn', { active: activeTab === 'status' }]"
        @click="activeTab = 'status'"
      >
        🌐 Core Metrics
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'peers' }]"
        @click="activeTab = 'peers'"
      >
        👥 Connected Peers ({{ network?.peers?.length || 0 }})
      </button>
    </div>

    <div v-if="loading" class="skeleton-wrapper">
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
        <section class="panel">
          <div class="panel-header">
            <h2 class="panel-title"><span class="panel-icon">🔧</span> System Configuration</h2>
          </div>
          <div class="panel-body">
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
                  <span v-if="stats.node?.synced" class="health-active">✓ Synced</span>
                  <span v-else class="health-standalone">Standalone Node</span>
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
          </div>
        </section>

        <section class="panel spacing-top-mobile">
          <div class="panel-header">
            <h2 class="panel-title"><span class="panel-icon">⛓</span> Consensus Height</h2>
          </div>
          <div class="panel-body">
            <div class="detail-container">
              <div class="detail-row">
                <span class="detail-label">Best Block Height</span>
                <span class="detail-value">
                  <NuxtLink :to="`/block/${stats.height}`">#{{ numFmt(stats.height) }}</NuxtLink>
                </span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Best Block Hash</span>
                <span class="detail-value mono">
                  <NuxtLink :to="`/block/${stats.best_hash}`">{{ stats.best_hash }}</NuxtLink>
                </span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Total Transactions</span>
                <span class="detail-value">{{ numFmt(stats.total_tx_count) }}</span>
              </div>
            </div>
          </div>
        </section>
      </div>

      <!-- ── Tab 2: Connected Peers List ── -->
      <div v-if="activeTab === 'peers' && network" class="peers-list-wrapper animate-fade">
        <section class="panel">
          <div class="panel-header">
            <h2 class="panel-title"><span class="panel-icon">👥</span> Peers List</h2>
          </div>
          <div class="panel-body">
            <div v-if="!network.peers || network.peers.length === 0" class="empty-state">
              No active peers connected. Nodes operate in standalone genesis mode.
            </div>
            <div v-else>
              <table class="grid-table">
                <thead>
                  <tr>
                    <th>Index</th>
                    <th>Peer Address (P2P)</th>
                    <th>Capabilities</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(peer, idx) in network.peers" :key="peer">
                    <td class="mono">#{{ idx + 1 }}</td>
                    <td class="mono">{{ peer }}</td>
                    <td>Full Node</td>
                    <td><span class="health-active">Active</span></td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </section>
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
  padding-bottom: var(--sp-sm);
}

.tab-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-family: inherit;
  font-size: 15px;
  font-weight: 600;
  padding: 8px 16px;
  cursor: pointer;
  border-radius: var(--r-sm);
  transition: background var(--t-fast), color var(--t-fast);
}

.tab-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tab-btn.active {
  background: hsla(258, 80%, 65%, 0.15);
  color: var(--brand-primary);
  border: 1px solid hsla(258, 80%, 65%, 0.25);
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

.health-standalone {
  background: hsla(38, 100%, 55%, 0.15);
  color: var(--warning);
  border: 1px solid hsla(38, 100%, 55%, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}
</style>
