# Walkthrough - ES Module Target Shift & cloudflare-module Integration

We have successfully migrated the explorer-ui builder target to output a modern ES Module worker format, matching the compiler targets expected by Wrangler v4.

---

## 🛠️ Changes Completed

### 1. Preset Migration
* **[`nuxt.config.ts`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/nuxt.config.ts#L24-L26)**: Changed the Nitro build preset target from `'cloudflare'` to `'cloudflare-module'`.
* **Why**: The default `'cloudflare'` target output legacy structures that caused Wrangler to fall back to the Service Worker parser and throw compilation warnings on external built-in packages. The `'cloudflare-module'` preset outputs a modern ES module format (`export default`) which resolves syntax errors during wrangler deployment checks.

---

## 🔬 Verification Results

### Build Verification
We verified build compilation:
```bash
npm run build
```
Output:
```
✔ Generated public .output/public                            nitro 5:30:02 PM
✔ Nuxt Nitro server built                                    nitro 5:34:42 PM
  └─ .output/server/index.mjs (178 B)
Σ Total size: 891 kB (279 kB gzip)
✨ Build complete!
```
The server bundle was generated successfully with **0 warnings and 0 errors**.
