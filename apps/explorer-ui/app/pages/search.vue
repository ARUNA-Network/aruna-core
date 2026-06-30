<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute, useRouter } from '#app'
import { search } from '~/services/api'
import Card from '~/components/ui/card/Card.vue'
import CardHeader from '~/components/ui/card/CardHeader.vue'
import CardTitle from '~/components/ui/card/CardTitle.vue'
import CardContent from '~/components/ui/card/CardContent.vue'
import SearchBar from '~/components/SearchBar.vue'

const route = useRoute()
const router = useRouter()

const query = ref(route.query.q as string || '')
const loading = ref(false)
const errorMsg = ref('')
const results = ref<any[]>([])

async function performSearch() {
  const q = query.value.trim()
  if (!q) return

  loading.value = true
  errorMsg.value = ''
  results.value = []

  try {
    const list = await search(q)
    results.value = list
    if (list && list.length === 1) {
      // Direct redirect if single match
      const match = list[0]
      if (match.kind === 'block') {
        router.replace(`/block/${encodeURIComponent(match.value)}`)
      } else if (match.kind === 'transaction') {
        router.replace(`/transaction/${encodeURIComponent(match.value)}`)
      } else if (match.kind === 'address') {
        router.replace(`/address/${encodeURIComponent(match.value)}`)
      }
    }
  } catch (err) {
    if (/^\d+$/.test(q)) {
      router.replace(`/block/${q}`)
    } else {
      errorMsg.value = (err as Error).message || 'Search failed.'
    }
  } finally {
    loading.value = false
  }
}

watch(() => route.query.q, (newQ) => {
  query.value = newQ as string || ''
  performSearch()
})

onMounted(() => {
  if (query.value) {
    performSearch()
  }
})
</script>

<template>
  <main class="container page-spacing">
    <Card>
      <CardHeader>
        <CardTitle><span class="panel-icon">🔍</span> Search Results</CardTitle>
      </CardHeader>
      <CardContent>
        <div class="search-box-wrapper mb-6">
          <SearchBar :placeholder="query || 'Search by block height, transaction hash, or address...'" />
        </div>

        <div v-if="loading" class="flex flex-col gap-3 py-6">
          <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
          <div class="h-6 bg-border/40 rounded animate-pulse w-full"></div>
        </div>
        <div v-else-if="errorMsg" class="error-state">
          <span class="error-icon">⚠️</span>
          <p>No results found</p>
          <span class="error-msg">{{ errorMsg }}</span>
        </div>
        <div v-else>
          <div v-if="results.length === 0" class="empty-state">
            Enter a search term to locate blocks, transactions, or addresses.
          </div>
          <div v-else class="flex flex-col gap-3">
            <h3 class="text-sm font-semibold text-text-secondary">Matching Records:</h3>
            <div class="list-container" role="list">
              <div
                v-for="item in results"
                :key="item.value"
                class="list-item flex items-center justify-between border-b border-border py-3"
              >
                <div class="flex flex-col">
                  <span class="text-xs font-semibold uppercase text-brand-primary">{{ item.kind }}</span>
                  <span class="mono text-sm">{{ item.value }}</span>
                </div>
                <NuxtLink
                  :to="`/${item.kind}/${encodeURIComponent(item.value)}`"
                  class="text-sm text-brand-primary font-semibold hover:underline"
                >
                  View Details →
                </NuxtLink>
              </div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  </main>
</template>

<style scoped>
.search-box-wrapper {
  max-width: 600px;
  margin: 0 auto var(--sp-lg) auto;
}
</style>
