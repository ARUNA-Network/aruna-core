# Walkthrough - Custom Domain Integration

We have mapped the custom domain `api.jojowi.web.id` to the Cloudflare Worker inside the `wrangler.toml` file to automate DNS and SSL configuration on Cloudflare.

---

## 🛠️ Changes Completed

### 1. wrangler.toml Update
* **[`wrangler.toml`](file:///home/coleallstar/Public/crypto-project/workers/explorer-api/wrangler.toml#L5-L8)**: Added the `[[routes]]` block to bind the custom domain `api.jojowi.web.id` directly to this worker:
  ```toml
  [[routes]]
  pattern = "api.jojowi.web.id"
  custom_domain = true
  ```

---

## 🔬 Verification Results

### Build Verification
We verified build compilation:
```bash
npx wrangler deploy --dry-run
```
Output:
```
Total Upload: 293.12 KiB / gzip: 56.67 KiB
No bindings found.
--dry-run: exiting now.
```
The configuration was validated and compiled successfully with **0 warnings and 0 errors**.
