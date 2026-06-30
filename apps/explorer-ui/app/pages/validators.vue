<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getNetwork } from '~/services/api'

const network = ref<any>(null)
const loading = ref(true)
const errorMsg = ref('')

async function fetchValidators() {
  loading.value = true
  errorMsg.value = ''
  try {
    const data = await getNetwork()
    network.value = data
  } catch (err) {
    errorMsg.value = (err as Error).message || 'Failed to load validators.'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchValidators()
})
</script>

<template>
  <main class="container page-spacing">
    <section class="panel">
      <div class="panel-header">
        <h1 class="panel-title"><span class="panel-icon">⛓</span> Active Validator Nodes</h1>
      </div>
      <div class="panel-body">
        <div v-if="loading" class="skeleton-wrapper">
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
        </div>
        <div v-else-if="errorMsg" class="error-state">
          <span class="error-icon">⚠️</span>
          <p>Failed to load validator nodes</p>
          <span class="error-msg">{{ errorMsg }}</span>
        </div>
        <div v-else>
          <table class="grid-table">
            <thead>
              <tr>
                <th>Validator</th>
                <th>Reward Address</th>
                <th>Stake Weight</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td class="mono">#1 (Local Node)</td>
                <td class="mono">
                  <NuxtLink :to="`/address/${network?.validators?.reward_address || 'sum1faucetaddressxxxxxxxxxxxxxxxxxxxxxxxxxx'}`">
                    {{ network?.validators?.reward_address || 'sum1faucetaddressxxxxxxxxxxxxxxxxxxxxxxxxxx' }}
                  </NuxtLink>
                </td>
                <td>10,000 ARU (Min Stake)</td>
                <td><span class="health-active">Active Validator</span></td>
              </tr>
            </tbody>
          </table>
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
</style>
