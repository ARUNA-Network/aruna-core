# ARUNA Network Genesis Allocation Specification (Sumatera Testnet)

This document specifies the initial allocation of ARUNA (ARU) tokens at the Genesis Block (Height 0) of the Sumatera Testnet. These parameters enforce the rules set by the ARUNA Network constitution.

## 1. Economic Summary

* **Max Supply:** 1,000,000,000 ARU
* **Genesis Allocated Supply:** 80,000,000 ARU (8.0% of Max Supply)
* **Vesting & Multi-sig Lockups:** All allocations are hardcoded and validated deterministically by the consensus engine.

| Allocation Type | Percentage | Amount (ARU) | Mapped Address | Purpose / Constraints |
| :--- | :--- | :--- | :--- | :--- |
| **Founder Allocation** | 1.5% | 15,000,000 | `sum1q5zs2pg9q5zs2pg9q5zs2pg9q5zs2pg9gw3snf` | Locked in a smart vesting contract. Linearly released over 48 months. No early unlock or governance privilege. |
| **Treasury** | 5.0% | 50,000,000 | `sumc1qszqgpqyqszqgpqyqszqgpqyqszqgpqypa49fy` | Contract address (`sumc1...`). Controlled entirely by on-chain governance; never founder funds. Used for grants, audits, and infrastructure. |
| **Premine - Pool 1** | 0.5% | 5,000,000 | `sum1qyqszqgpqyqszqgpqyqszqgpqyqszqgpe6sslr` | Testnet Rewards & Node bootstrapping incentives. |
| **Premine - Pool 2** | 0.5% | 5,000,000 | `sum1qgpqyqszqgpqyqszqgpqyqszqgpqyqszg7k454` | Initial network infrastructure bootstrapping. |
| **Premine - Pool 3** | 0.5% | 5,000,000 | `sum1qvpsxqcrqvpsxqcrqvpsxqcrqvpsxqcrfwh575` | Liquidity experiments & Security bounties. |
| **Total** | **8.0%** | **80,000,000** | — | — |

---

## 2. Allocation Bech32m Encodings

The genesis addresses use the `sum` prefix for user accounts and `sumc` prefix for smart contracts. All are left-padded with zero bytes to 32 bytes for EVM compatibility and stored in RocksDB state.
