<script setup lang="ts">
import type { AddressData } from '~/types'
import { numFmt, microAruToAru } from '~/utils/format'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'

defineProps<{
  address: string
  addressData: AddressData
}>()
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle><span class="panel-icon">💳</span> Account Metrics</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="detail-container">
        <div class="detail-row">
          <span class="detail-label">Address</span>
          <span class="detail-value mono text-xs leading-5">{{ address }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Balance</span>
          <span class="detail-value text-glow font-bold">{{ microAruToAru(addressData.balance) }} ARU</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Nonce</span>
          <span class="detail-value">{{ numFmt(addressData.nonce) }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Last Updated Block</span>
          <span class="detail-value">
            <NuxtLink :to="`/block/${addressData.updated_at_block}`">#{{ numFmt(addressData.updated_at_block) }}</NuxtLink>
          </span>
        </div>
      </div>
    </CardContent>
  </Card>
</template>
