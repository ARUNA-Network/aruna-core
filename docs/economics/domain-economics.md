# Economic Simulation: Domain Naming Service (Aruna Names)

This document defines the pricing, registration tiers, and fee distribution model for the native ARUNA domain naming service (`.aruna`).

## 1. Domain Registration Pricing
To prevent domain squatting and ensure that premium names are priced appropriately, registration fees are categorized by name length. All registration fees are paid in **ARU** and are subject to annual renewal:

| Character Length | Category | Annual Registration Fee (ARU) |
| :--- | :--- | :--- |
| **3 characters** (e.g. `aru.aruna`) | Premium Gold | $100 \text{ ARU}$ |
| **4 characters** (e.g. `cole.aruna`) | Premium Silver | $50 \text{ ARU}$ |
| **5+ characters** (e.g. `alox12.aruna`)| Standard | $10 \text{ ARU}$ |

---

## 2. Fee Distribution & Routing
1. **Treasury Routing:** 100% of domain registration and renewal fees are sent directly to the network **Treasury** account.
2. **Future Validator Split:** Once the Jawa Mainnet launches, governance may approve splitting domain revenue:
   * **90% to Treasury** (for developer grants and audits).
   * **10% to Validator Pool** (distributed as block execution bonuses to active stakers).

---

## 3. Lifecycle & Renewal Dynamics
* **Expiration:** Domains are registered for 1 year (365 days).
* **Grace Period:** If a domain is not renewed by its expiration block, it enters a **30-day grace period**. During this period, the domain is inactive (does not resolve to the address) but remains locked, allowing the original owner to renew it.
* **Release:** If the grace period expires, the domain is purged from RocksDB storage and becomes available for public registration.
