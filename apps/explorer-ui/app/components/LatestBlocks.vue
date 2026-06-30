<script setup lang="ts">
import type { Block } from '~/types'
import { numFmt, timestamp, shortHash } from '~/utils/format'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import Table from '~/components/ui/table/Table.vue'
import TableHeader from '~/components/ui/table/TableHeader.vue'
import TableBody from '~/components/ui/table/TableBody.vue'
import TableRow from '~/components/ui/table/TableRow.vue'
import TableHead from '~/components/ui/table/TableHead.vue'
import TableCell from '~/components/ui/table/TableCell.vue'

defineProps<{
  blocks: Block[]
  loading: boolean
}>()
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle><span class="panel-icon">📦</span> Recent Blocks</CardTitle>
    </CardHeader>
    <CardContent>
      <div v-if="loading && blocks.length === 0" class="flex flex-col gap-3">
        <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
        <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
        <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
      </div>
      <div v-else>
        <div v-if="blocks.length === 0" class="text-center py-6 text-text-muted">
          No blocks found.
        </div>
        <div v-else>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Height</TableHead>
                <TableHead>Hash</TableHead>
                <TableHead>Timestamp</TableHead>
                <TableHead>Transactions</TableHead>
                <TableHead>Difficulty</TableHead>
                <TableHead>Nonce</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-for="block in blocks" :key="block.hash">
                <TableCell>
                  <NuxtLink :to="`/block/${block.height}`">#{{ numFmt(block.height) }}</NuxtLink>
                </TableCell>
                <TableCell class="mono">
                  <NuxtLink :to="`/block/${block.hash}`">{{ shortHash(block.hash) }}</NuxtLink>
                </TableCell>
                <TableCell>{{ timestamp(block.timestamp) }}</TableCell>
                <TableCell>{{ numFmt(block.tx_count) }}</TableCell>
                <TableCell>{{ numFmt(block.difficulty) }}</TableCell>
                <TableCell>{{ numFmt(block.nonce) }}</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </div>
    </CardContent>
  </Card>
</template>
