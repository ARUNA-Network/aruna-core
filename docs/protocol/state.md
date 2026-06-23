# Protocol Validation: State Specification

This document details the state representation, database backend, and state transition logic for the ARUNA Network core ledger.

## 1. Account State Structure
ARUNA uses an account-based state model. The global state is a mapping of addresses to account states. Each account state contains:
* **Address (32 bytes / `[u8; 32]`):** The decoded public key hash of the account.
* **Balance (8 bytes / `u64`):** The account balance in micro-ARU.
* **Nonce (8 bytes / `u64`):** The transaction counter.
* **Code Hash (32 bytes / `[u8; 32]`):** The hash of the deployed contract bytecode (or `[0u8; 32]` if a standard account).
* **Storage Root (32 bytes / `[u8; 32]`):** The root hash of the account's internal key-value storage (or `[0u8; 32]` if empty).

## 2. Storage Backend (RocksDB)
* **Standard:** **RocksDB**.
* **Rationale:** RocksDB is an embedded key-value store that provides high write throughput, fast point lookups, and excellent stability on both x86_64 and ARM64 platforms. It operates directly inside the node process, removing network socket latency.
* **Key-Value Layout:**
  * **Account State Prefix (`a`):**
    * *Key:* `a || Address (32 bytes)`
    * *Value:* Serialized account state (Balance, Nonce, Code Hash, Storage Root).
  * **Contract Code Prefix (`c`):**
    * *Key:* `c || CodeHash (32 bytes)`
    * *Value:* Raw EVM bytecode.
  * **Contract Storage Prefix (`s`):**
    * *Key:* `s || Address (32 bytes) || StorageKey (32 bytes)`
    * *Value:* StorageValue (32 bytes).

## 3. State Transitions
Ledger mutations must occur deterministically during transaction execution.

### Transition Pipeline:
```
State_N (Global Database)
      │
      ▼
Verify Nonce & Balance (Tx validation)
      │
      ▼
Deduct Fees & Value (Sender account update)
      │
      ▼
Execute EVM / Staking (Contract execution / state modification)
      │
      ▼
Credit Value & Rewards (Recipient, Miner, and Validator accounts update)
      │
      ▼
State_N+1 (Global Database Write)
```

### Determinism Rule:
* Floating-point arithmetic is **strictly forbidden** inside the state transition engine. All balances, fee calculations, and gas multipliers must use fixed-point integer mathematics.
* Any compilation target mismatch or divergent output on a block transition between an ARM64 phone and an x86_64 PC is treated as a critical consensus failure.
