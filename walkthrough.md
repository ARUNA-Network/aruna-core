# Walkthrough - Nuxt 4 Migration, Pinia, & shadcn-vue for explorer-ui

We have successfully migrated the static `explorer-ui` to an enterprise-grade **Nuxt 4** framework with **Pinia** state management and **shadcn-vue** primitive components.

---

## 🛠️ Changes Completed

### 1. Configuration & Dependency Layer
* **[`package.json`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/package.json)**: Added `@pinia/nuxt`, `@nuxtjs/tailwindcss`, `shadcn-nuxt`, `radix-vue`, `class-variance-authority`, `clsx`, `tailwind-merge`, and `@lucide/vue`.
* **[`nuxt.config.ts`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/nuxt.config.ts)**: Configured Nuxt modules (`@pinia/nuxt`, `@nuxtjs/tailwindcss`, `shadcn-nuxt`), components lookup mapping, and runtime options.
* **[`tailwind.config.js`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/tailwind.config.js)**: Configured brand palettes (`brand-primary`, `brand-secondary`, `brand-glow`), font metrics (`Outfit`, `JetBrains Mono`), and shadcn styles.
* **[`app/styles/main.css`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/app/styles/main.css#L1-L41)**: Prepended `@tailwind` base directives and mapped custom variables.

### 2. State Management Layer (Pinia)
We placed all transaction, block, and network caching logic under **`app/stores/`**:
* **[`network.ts`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/app/stores/network.ts)**: Holds network status stats and connected peer connections.
* **[`block.ts`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/app/stores/block.ts)**: Handles the latest block, details, and historical block lists.
* **[`tx.ts`](file:///home/coleallstar/Public/crypto-project/apps/explorer-ui/app/stores/tx.ts)**: Holds account balance history and transaction hashes.

*All data fetches route exclusively through `app/services/api.ts` into these Pinia stores.*

### 3. UI Primitives (shadcn-vue under `/app/components/ui/`)
We implemented custom, lightweight, and modern UI components:
* **`Button`**: Radical primitive supporting secondary/outline/destructive variants.
* **`Card`**: Shell structure (Card, CardHeader, CardTitle, CardContent).
* **`Table`**: Flexible grid cells (Table, TableHeader, TableRow, TableHead, TableBody, TableCell).
* **`Badge`**: Status tokens for transaction details.

### 4. Page Integration Refactor (`app/pages/`)
We refactored all core routes to read re-actively from Pinia stores and render clean shadcn components:
* `index.vue` (Home page)
* `blocks.vue` (Paginated block lists)
* `block/[id].vue` (Detailed block view)
* `transactions.vue` (Lookup landing)
* `transaction/[hash].vue` (Confirmed transaction details)
* `address/[address].vue` (Account details and logs)
* `network.vue` (Diagnostic statistics and active peers list)
* `validators.vue` (Active validators stack)
* `stats.vue` (Supply details)

---

## 🔬 Verification Results

### Build Compilation
We successfully verified the entire compilation of Nuxt 4, Pinia, and TailwindCSS:
```bash
npm run build
```
Output:
```
✔ Server built in 3593ms
✔ Generated public dist
✔ Nuxt Nitro server built
  └─ dist/_worker.js/index.js (251 B) (187 B)
Σ Total size: 778 kB (239 kB gzip)
✨ Build complete!
```
The server bundle was generated successfully with **0 warnings and 0 errors**.
