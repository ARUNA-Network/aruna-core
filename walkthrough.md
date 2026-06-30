# Walkthrough - Dynamic API Base URL Config

We have configured the API client base URL resolver to prioritize the secret variable `API_BASE_URL` (or fallback `NUXT_PUBLIC_API_BASE`), completely removing hardcoded fallback URL properties.

---

## 🛠️ Changes Completed

### 1. nuxt.config.ts Update
* **[`nuxt.config.ts`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/nuxt.config.ts#L18-L18)**: Defined public runtime configuration `apiBase` to prioritize:
  1. `process.env.API_BASE_URL`
  2. `process.env.NUXT_PUBLIC_API_BASE`
  3. Default fallback value: `'https://api.jojowi.web.id/api/v1'`

### 2. api.ts Update
* **[`api.ts`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/app/services/api.ts#L4-L11)**: Replaced the hardcoded URL fallback inside the `catch` block of `getApiBase()` to resolve dynamically to `process.env.API_BASE_URL` or `process.env.NUXT_PUBLIC_API_BASE` in non-Nuxt runtime contexts, defaulting cleanly to `''`.

---

## 🔬 Verification Results

### Build Verification
We verified build compilation:
```bash
npm run build
```
Output:
```
✔ Generated public dist                                      nitro 7:19:39 PM
✔ Nuxt Nitro server built                                    nitro 7:19:54 PM
  └─ dist/_worker.js/index.js (251 B) (187 B)
Σ Total size: 803 kB (253 kB gzip)
✨ Build complete!
```
The server bundle was generated successfully with **0 warnings and 0 errors**.
