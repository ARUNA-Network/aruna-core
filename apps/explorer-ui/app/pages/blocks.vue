<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useRoute, useRouter } from '#app'
import { storeToRefs } from 'pinia'
import { useBlockStore } from '~/stores/block'
import { numFmt, timestamp, shortHash } from '~/utils/format'

// UI Primitives
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import Button from '~/components/ui/button/Button.vue'
import Table from '~/components/ui/table/Table.vue'
import TableHeader from '~/components/ui/table/TableHeader.vue'
import TableBody from '~/components/ui/table/TableBody.vue'
import TableRow from '~/components/ui/table/TableRow.vue'
import TableHead from '~/components/ui/table/TableHead.vue'
import TableCell from '~/components/ui/table/TableCell.vue'

const limit = 20
const blockStore = useBlockStore()
const { blocksPage: blocks, loading, error: errorMsg } = storeToRefs(blockStore)

const route = useRoute()
const router = useRouter()
const currentPage = ref(Number(route.query.page) || 1)

async function fetchBlocks() {
  const offset = (currentPage.value - 1) * limit
  await blockStore.fetchBlocksPage(limit, offset)
}

watch(() => route.query.page, (newPage) => {
  currentPage.value = Number(newPage) || 1
  fetchBlocks()
})

onMounted(() => {
  fetchBlocks()
})

function navigateToPage(page: number) {
  router.push({ query: { page } })
}
</script>

<template>
  <main class="container page-spacing">
    <Card>
      <CardHeader>
        <CardTitle><span class="panel-icon">📦</span> Blocks</CardTitle>
      </CardHeader>
      <CardContent>
        <div v-if="loading && blocks.length === 0" class="skeleton-wrapper">
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
        </div>
        <div v-else-if="errorMsg" class="error-state">
          <span class="error-icon">⚠️</span>
          <p>Failed to load blocks</p>
          <span class="error-msg">{{ errorMsg }}</span>
        </div>
        <div v-else>
          <div v-if="blocks.length === 0" class="empty-state">
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

            <!-- Pagination Bar -->
            <div class="pagination-bar">
              <Button
                :disabled="currentPage <= 1"
                @click="navigateToPage(currentPage - 1)"
                variant="secondary"
              >
                ← Previous
              </Button>
              <span class="page-indicator">Page {{ currentPage }}</span>
              <Button
                :disabled="blocks.length < limit"
                @click="navigateToPage(currentPage + 1)"
                variant="secondary"
              >
                Next →
              </Button>
            </div>
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

.pagination-bar {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: var(--sp-md);
  margin-top: var(--sp-lg);
  padding-top: var(--sp-md);
  border-top: 1px solid var(--border);
}

.page-indicator {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}
</style>
