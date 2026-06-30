import { defineStore } from 'pinia'
import { getStatus, getNetwork } from '~/services/api'
import type { Stats, NetworkData } from '~/types'

export const useNetworkStore = defineStore('network', {
  state: () => ({
    stats: null as Stats | null,
    network: null as NetworkData | null,
    loading: false,
    error: ''
  }),
  actions: {
    async fetchNetworkData() {
      this.loading = true
      this.error = ''
      try {
        const [statsData, networkData] = await Promise.all([
          getStatus(),
          getNetwork()
        ])
        this.stats = statsData
        this.network = networkData
      } catch (err) {
        this.error = (err as Error).message || 'Failed to load network diagnostics.'
      } finally {
        this.loading = false
      }
    }
  }
})
