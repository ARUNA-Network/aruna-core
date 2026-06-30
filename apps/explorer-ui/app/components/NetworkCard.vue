<script setup lang="ts">
import type { Stats } from '~/types'
import { numFmt, timeAgo } from '~/utils/format'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'

defineProps<{
  stats: Stats | null
}>()
</script>

<template>
  <Card id="network-status-panel" aria-label="Network Status">
    <CardHeader>
      <CardTitle>
        <span class="panel-icon">🌐</span> Network Status
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="stats-grid" id="stats-grid" role="region" aria-live="polite">
        <div class="stat-card">
          <div class="stat-icon">📦</div>
          <div :class="['stat-value', { skeleton: !stats }]">
            {{ stats ? numFmt(stats.height) : '—' }}
          </div>
          <div class="stat-label">Block Height</div>
        </div>
        <div class="stat-card">
          <div class="stat-icon">⚡</div>
          <div :class="['stat-value', { skeleton: !stats }]">
            {{ stats ? numFmt(stats.total_tx_count) : '—' }}
          </div>
          <div class="stat-label">Total Transactions</div>
        </div>
        <div class="stat-card">
          <div class="stat-icon">⏱</div>
          <div :class="['stat-value', { skeleton: !stats }]">
            {{ stats ? timeAgo(stats.last_block_time) : '—' }}
          </div>
          <div class="stat-label">Last Block Time</div>
        </div>
        <div class="stat-card">
          <div class="stat-icon">👥</div>
          <div :class="['stat-value', { skeleton: !stats }]">
            {{ stats?.node ? numFmt(stats.node.peer_count) : '0' }}
          </div>
          <div class="stat-label">Connected Peers</div>
        </div>
      </div>
    </CardContent>
  </Card>
</template>
