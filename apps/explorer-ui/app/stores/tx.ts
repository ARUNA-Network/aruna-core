import { defineStore } from 'pinia'
import { getTransaction, getAddress } from '~/services/api'
import type { Transaction, AddressData } from '~/types'

export const useTxStore = defineStore('tx', {
  state: () => ({
    currentTx: null as Transaction | null,
    addressDetails: null as AddressData | null,
    loading: false,
    error: '',
    addressError: ''
  }),
  actions: {
    async fetchTransactionDetails(hash: string) {
      this.loading = true
      this.error = ''
      try {
        const tx = await getTransaction(hash)
        this.currentTx = tx
      } catch (err) {
        this.error = (err as Error).message || 'Failed to load transaction details.'
        this.currentTx = null
      } finally {
        this.loading = false
      }
    },
    async fetchAddressDetails(address: string, limit = 20, offset = 0) {
      this.loading = true
      this.addressError = ''
      try {
        const data = await getAddress(address, limit, offset)
        this.addressDetails = data
      } catch (err) {
        this.addressError = (err as Error).message || 'Failed to load address details.'
        this.addressDetails = null
      } finally {
        this.loading = false
      }
    }
  }
})
