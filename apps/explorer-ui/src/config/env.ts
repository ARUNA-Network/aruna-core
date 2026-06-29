export const env = {
  // Connect to the new TypeScript Worker API running on Cloudflare Edge by default
  API_BASE: (window as any).ARUNA_API_URL || 'http://127.0.0.1:8787/api/v1',
  REFRESH_INTERVAL_MS: 12000, // 12 seconds
  MICRO_ARU: 1_000_000,
};
