import { defineNuxtConfig } from 'nuxt/config'

export default defineNuxtConfig({
  future: {
    compatibilityVersion: 4,
  },
  modules: [
    '@pinia/nuxt',
    '@nuxtjs/tailwindcss',
    'shadcn-nuxt'
  ],
  shadcn: {
    prefix: '',
    componentDir: './app/components/ui'
  },
  runtimeConfig: {
    public: {
      apiBase: process.env.API_BASE_URL || process.env.NUXT_PUBLIC_API_BASE || 'https://api.jojowi.web.id/api/v1',
      networkName: process.env.NUXT_PUBLIC_NETWORK || process.env.NETWORK || 'Sumatera Testnet',
      chainId: process.env.NUXT_PUBLIC_CHAIN_ID || process.env.CHAIN_ID || 'sumatera-testnet',
      refreshIntervalMs: 12000,
      microAru: 1000000
    }
  },
  css: ['~/styles/main.css'],
  nitro: {
    preset: 'cloudflare-pages'
  }
})
