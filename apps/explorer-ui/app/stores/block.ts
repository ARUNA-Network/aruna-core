import { defineStore } from 'pinia'
import { getBlock, getLatestBlock, getBlocks } from '~/services/api'
import type { Block } from '~/types'

export const useBlockStore = defineStore('block', {
  state: () => ({
    latestBlock: null as Block | null,
    currentBlock: null as Block | null,
    blocksPage: [] as Block[],
    loading: false,
    error: '',
    latestBlockError: '',
    latestTxsError: ''
  }),
  actions: {
    async fetchLatestBlock() {
      this.loading = true
      this.latestBlockError = ''
      this.latestTxsError = ''
      try {
        const block = await getLatestBlock()
        this.latestBlock = block
      } catch (err) {
        this.latestBlockError = (err as Error).message || 'Failed to load latest block.'
        this.latestTxsError = (err as Error).message || 'Failed to load latest transactions.'
      } finally {
        this.loading = false
      }
    },
    async fetchBlockDetails(heightOrHash: string | number) {
      this.loading = true
      this.error = ''
      try {
        const block = await getBlock(heightOrHash)
        this.currentBlock = block
      } catch (err) {
        this.error = (err as Error).message || 'Failed to load block details.'
        this.currentBlock = null
      } finally {
        this.loading = false
      }
    },
    async fetchBlocksPage(limit: number, offset: number) {
      this.loading = true
      this.error = ''
      try {
        const list = await getBlocks(limit, offset)
        this.blocksPage = list
      } catch (err) {
        this.error = (err as Error).message || 'Failed to load blocks list.'
        this.blocksPage = []
      } finally {
        this.loading = false
      }
    }
  }
})
