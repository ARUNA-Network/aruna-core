# ARUNA NETWORK

## MASTER PRD

### Volume 11 — Product Suite PRD

---

# PRODUCT ECOSYSTEM OVERVIEW

The ARUNA ecosystem consists of the following official applications and core services:
1. **ARUNA Wallet** (`Aruna Wallet`)
2. **ARUNA Miner** (`Aruna Mine`)
3. **ARUNA Explorer** (`Aruna Scan`)
4. **ARUNA Validator Hub** (`Aruna Validator`)
5. **ARUNA DEX** (`Aruna Swap`)
6. **ARUNA Domains** (`Aruna Names`)
7. **ARUNA Governance** (`Aruna Gov`)
8. **ARUNA Launchpad** (`Aruna Launch`)
9. **ARUNA AI** (`Aruna AI`)

---

# PRODUCT ROADMAP PHASES

The rollout plan is divided into structured product launches:
* **Phase 1:** Wallet + Explorer + Miner (Sumatera Testnet launchpad).
* **Phase 2:** Validator Hub + Governance Portal.
* **Phase 3:** DEX + Domain Naming Service.
* **Phase 4:** Launchpad + Bridge.

---

# PRODUCT 1 — ARUNA WALLET (`Aruna Wallet`)

* **Purpose:** Official wallet client for storing ARU, transfers, staking, voting, domain management, and native DEX integrations.
* **Platforms:** Android, iOS, Windows, Linux, macOS.
* **Technology:** Flutter.
* **Core Features:**
  * Wallet creation, BIP39 seed phrase import, and Watch wallets.
  * Multi-wallet and multi-account management.
  * Direct sends and receives with QR support.
  * Native Staking (stake, unstake, validator delegation/selection).
  * Native Governance (proposal lookup and voting).
  * Biometric login, PIN locks, and offline seed backup.

---

# PRODUCT 2 — ARUNA MINER (`Aruna Mine`)

* **Purpose:** CPU/ARM-optimized mining interface for Android, Raspberry Pi, Mini PC, and x86 machines.
* **Platforms:** Android, Linux, Windows, macOS.
* **Core Features:**
  * Dashboard displaying Hashrate, accepted shares, device temperature, and battery power draw.
  * Official pool mining, community pools, and solo mining.
  * Mandatory device safety (thermal protections, battery drop alerts, CPU usage tuning).
  * Android-specific configurations: Screen-off mining, WiFi-only constraints, and charging-only toggles.
  * Community leaderboards, mining ranks, and milestones.

---

# PRODUCT 3 — ARUNA EXPLORER (`Aruna Scan`)

* **Purpose:** Public blockchain explorer and indexer dashboard.
* **Stack:** Nuxt (Frontend) + Rust (Backend) + PostgreSQL (Database).
* **Core Features:**
  * Search bar resolving addresses, tx hashes, block numbers, validators, and domains.
  * Realtime metrics: block height, adjustment difficulty, hashrate, active validators, circulating supply.
  * Address dashboard displaying balances, transaction list, staking states, and registered domains.
  * Validator ranking: commission rate, staked weight, uptime, and blocks signed.
  * Token explorer showing ARC-20 contracts, LP pairs, and holders.
  * Governance board displaying proposals, voting splits, and treasury transactions.

---

# PRODUCT 4 — ARUNA VALIDATOR HUB (`Aruna Validator`)

* **Purpose:** Validator management portal.
* **Stack:** Nuxt + Rust API.
* **Core Features:**
  * Performance dashboard displaying stake weight, delegation accounts, rewards earned, and APR.
  * Operation controllers (start, stop, maintenance modes).
  * Governance reviews and voting tools.
  * Detailed node analytics (uptime, missed blocks, block proposal rates).

---

# PRODUCT 5 — ARUNA DEX (`Aruna Swap`)

* **Purpose:** Native decentralized swap and liquidity protocol.
* **Stack:** Solidity Smart Contracts + Wallet Integration.
* **Core Features:**
  * Swaps: `ARU` ↔ `ARC-20` and `ARC-20` ↔ `ARC-20`.
  * Liquidity provision (add/remove liquidity, LP share distribution, fee rewards).
  * Analytics panel showing TVL, trade volume, and fee collection.
  * Token discovery (verified, new, and trending tokens).

---

# PRODUCT 6 — ARUNA DOMAINS (`Aruna Names`)

* **Purpose:** Human-readable naming and identity service (`nama.aruna`).
* **Core Features:**
  * Registration, renewal, and transfer of domains.
  * Forward resolution (`example.aruna` → address) and reverse resolution (address → `example.aruna`).
  * Full wallet integration to display `example.aruna` instead of raw Bech32m hashes.

---

# PRODUCT 7 — ARUNA GOVERNANCE (`Aruna Gov`)

* **Purpose:** Staking-vote portal for community governance.
* **Core Features:**
  * Proposal creation (Governance, Treasury, Protocol upgrades).
  * Staking weight and validator reviews.
  * Live treasury dashboards showing total income, expenditures, and budgets.

---

# PRODUCT 8 & 9 — LAUNCHPAD (`Aruna Launch`) & ARUNA AI (`Aruna AI`)

* **Aruna Launch (Future):** Simple IDO setup with token creation, vesting logic, and liquidity bootstrapping.
* **Aruna AI (Future Vision):** AI-powered ecosystem layer for autonomous agents, developer tooling, governance audits, and treasury analysis.

---

# UX & BRAND PHILOSOPHY

* **UX Standards:** Simple, fast, mobile-first, accessible, and localized.
* **Languages:** **Bahasa Indonesia** and **English** natively.
* **Visual Identity:** Indonesia First, modern, trustworthy, open, and community-driven.

---

# SUCCESS METRICS

* **Wallet:** 10,000+ installs.
* **Miner:** 1,000+ active miners.
* **Explorer:** 99.9% uptime.
* **Validators:** 50+ active validators.
* **DEX:** $100,000+ TVL.
* **Domains:** 10,000+ domains registered.

End of Volume 11.
