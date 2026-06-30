# Walkthrough - Explorer API Router Prefix Compatibility

We have added support for both the `/explorer/v1/` and `/api/v1/` route prefixes in the Cloudflare Worker router. This enables the frontend explorer-ui calling `/explorer/v1/status` to match the routes in the worker and respond correctly.

---

## 🛠️ Changes Completed

### 1. Main Entrypoint Router Update
* **[`index.ts`](file:///home/coleallstar/Public/crypto-project/workers/explorer-api/src/index.ts#L48-L76)**:
  * Expanded route distribution condition checks (`isStatus`, `isBlocks`, `isTransaction`, `isAddress`, `isSearch`, `isNetwork`) to support both `/api/v1` and `/explorer/v1` path prefixes.

### 2. Route Handlers Update
* **[`blocks.ts`](file:///home/coleallstar/Public/crypto-project/workers/explorer-api/src/routes/blocks.ts#L7-L85)**: Added matches for `/explorer/v1/blocks`, `/explorer/v1/block/latest`, and regex support `/(?:api|explorer)/v1/block/` for height and hash endpoints.
* **[`addresses.ts`](file:///home/coleallstar/Public/crypto-project/workers/explorer-api/src/routes/addresses.ts#L7-L7)**: Replaced path matching regex with `/(?:api|explorer)/v1/address/`.
* **[`transactions.ts`](file:///home/coleallstar/Public/crypto-project/workers/explorer-api/src/routes/transactions.ts#L7-L7)**: Replaced path matching regex with `/(?:api|explorer)/v1/transaction/`.

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
The worker bundles build successfully with **0 warnings and 0 errors**.
