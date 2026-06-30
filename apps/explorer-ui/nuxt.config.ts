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
      apiBase: process.env.NUXT_PUBLIC_API_BASE || 'https://api.jojowi.web.id/api/v1',
      refreshIntervalMs: 12000,
      microAru: 1000000
    }
  },
  css: ['~/styles/main.css'],
  nitro: {
    preset: 'cloudflare-module'
  }
})
