<script setup lang="ts">
import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useNetworkStore } from '~/stores/network'

// UI Primitives
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import Badge from '~/components/ui/badge/Badge.vue'
import Table from '~/components/ui/table/Table.vue'
import TableHeader from '~/components/ui/table/TableHeader.vue'
import TableBody from '~/components/ui/table/TableBody.vue'
import TableRow from '~/components/ui/table/TableRow.vue'
import TableHead from '~/components/ui/table/TableHead.vue'
import TableCell from '~/components/ui/table/TableCell.vue'

const networkStore = useNetworkStore()
const { network, loading, error: errorMsg } = storeToRefs(networkStore)

onMounted(() => {
  networkStore.fetchNetworkData()
})
</script>

<template>
  <main class="container page-spacing">
    <Card>
      <CardHeader>
        <CardTitle><span class="panel-icon">⛓</span> Active Validator Nodes</CardTitle>
      </CardHeader>
      <CardContent>
        <div v-if="loading && !network" class="skeleton-wrapper">
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
        </div>
        <div v-else-if="errorMsg" class="error-state">
          <span class="error-icon">⚠️</span>
          <p>Failed to load validator nodes</p>
          <span class="error-msg">{{ errorMsg }}</span>
        </div>
        <div v-else>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Validator</TableHead>
                <TableHead>Reward Address</TableHead>
                <TableHead>Stake Weight</TableHead>
                <TableHead>Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow>
                <TableCell class="mono">#1 (Local Node)</TableCell>
                <TableCell class="mono">
                  <NuxtLink :to="`/address/${network?.validators?.reward_address || 'sum1faucetaddressxxxxxxxxxxxxxxxxxxxxxxxxxx'}`">
                    {{ network?.validators?.reward_address || 'sum1faucetaddressxxxxxxxxxxxxxxxxxxxxxxxxxx' }}
                  </NuxtLink>
                </TableCell>
                <TableCell>10,000 ARU (Min Stake)</TableCell>
                <TableCell><Badge variant="success">Active Validator</Badge></TableCell>
              </TableRow>
            </TableBody>
          </Table>
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
</style>
