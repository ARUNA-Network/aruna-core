# Protocol Specification: Genesis Block & Allocations

This document defines the parameters, initial token allocations, and binary header specifications for the ARUNA Network Genesis Block (Block Height 0).

## 1. Genesis Block Header Specification

The genesis block header fields are defined as follows:

| Field | Type | Value | Description |
| :--- | :--- | :--- | :--- |
| **version** | `u32` | `1` | Initial protocol version |
| **prev_block_hash** | `Hash` | `0x00000000...00` (32 bytes) | Zero parent hash |
| **merkle_root** | `Hash` | Calculated Root | BLAKE3 Merkle root of the genesis allocations |
| **timestamp** | `u64` | `1782252000` | POSIX timestamp (June 23, 2026, 22:00:00 UTC) |
| **difficulty** | `Difficulty` | `0x1e0ffff0` (nBits) | Low initial difficulty target for CPU/ARM bootstrap |
| **nonce** | `u64` | `0` | Default proof-of-work nonce |
| **validator_root**| `Hash` | `0x00000000...00` (32 bytes) | Zero root (no staker signatures exist for block 0) |
| **treasury_root** | `Hash` | Calculated Root | BLAKE3 hash of the initial treasury allocation transactions |

---

## 2. Token Allocation Rules (Genesis Era)

ARUNA enforces a strict **1,000,000,000 ARU** maximum supply cap. The genesis block pre-allocates **8.0%** of the supply, with the remaining **92.0%** minted dynamically via block rewards.

* **Premine (1.5%):** 15,000,000 ARU ($1.5 \times 10^{13}$ micro-ARU) for network bootstrapping, faucets, and liquidity experiments.
* **Treasury (5.0%):** 50,000,000 ARU ($5.0 \times 10^{13}$ micro-ARU) locked in governance contracts.
* **Founder Allocation (1.5%):** 15,000,000 ARU ($1.5 \times 10^{13}$ micro-ARU) locked in a vesting contract (48-month monthly linear vesting, zero voting or reward privileges during lockup).

---

## 3. Network Allocations (Sumatera Testnet)

For the Sumatera Testnet, the initial accounts are mapped to the following Bech32m-decoded addresses (20-byte public key hashes left-padded to 32 bytes):

### A. Bootstrap Staking Nodes (Premine)
* **Bootstrap Validator 1:**
  * *Address:* `sum1qyqszqgpqyqszqgpqyqszqgpqyqszqgpe6sslr`
  * *Allocation:* 5,000,000 ARU (to establish initial PoS weight).
* **Bootstrap Validator 2:**
  * *Address:* `sum1qgpqyqszqgpqyqszqgpqyqszqgpqyqszg7k454`
  * *Allocation:* 5,000,000 ARU.
* **Faucet and Testing Fund:**
  * *Address:* `sum1qvpsxqcrqvpsxqcrqvpsxqcrqvpsxqcrfwh575`
  * *Allocation:* 5,000,000 ARU.

### B. Network Treasury
* **Treasury Governance Contract:**
  * *Address:* `sumc1qszqgpqyqszqgpqyqszqgpqyqszqgpqypa49fy`
  * *Allocation:* 50,000,000 ARU.

### C. Founder Lockup
* **Founder Vesting Contract:**
  * *Address:* `sum1q5zs2pg9q5zs2pg9q5zs2pg9q5zs2pg9gw3snf`
  * *Allocation:* 15,000,000 ARU.

---

## 4. Initialization Validation
1. **Verification of Roots:** Nodes verify that the `merkle_root` matches the hash root derived from the 5 initial transactions corresponding to the allocations above.
2. **First Transition (Block 1):** Block 1 difficulty adjustments are calculated using the genesis timestamp. The first block reward (25 ARU) is distributed exactly according to the 70%/25%/5% split rules.
