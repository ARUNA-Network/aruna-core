# Protocol Validation: State & Storage Specification

This document details the state representation, database backend, key-value schemas, and state transition logic for the ARUNA Network core ledger.

## 1. Account State Structure
ARUNA uses an account-based state model. The global state is a mapping of addresses to account states. Each account state contains:
* **Address (32 bytes / `[u8; 32]`):** The decoded public key hash of the account (left-padded with 12 zero bytes).
* **Balance (8 bytes / `u64`):** The account balance in micro-ARU.
* **Nonce (8 bytes / `u64`):** The transaction counter.
* **Code Hash (32 bytes / `[u8; 32]`):** The hash of the deployed contract bytecode (or `[0u8; 32]` if a standard user account).
* **Storage Root (32 bytes / `[u8; 32]`):** The root hash of the account's internal key-value storage (or `[0u8; 32]` if empty).

---

## 2. Storage Backend (RocksDB) Key-Value Schema
To ensure determinism and low lookup latency, all ledger states are stored in RocksDB. Keys are prefixed with a single byte identifier.

### A. Prefix Catalog

| Prefix | Byte | Hex | Purpose |
| :--- | :--- | :--- | :--- |
| `a` | 97 | `0x61` | Account State Table |
| `c` | 99 | `0x63` | Contract Bytecode Table |
| `s` | 115| `0x73` | Contract Storage Table |
| `h` | 104| `0x68` | Block Header Index Table |
| `d` | 100| `0x64` | Block Body (Transactions) Table |
| `t` | 116| `0x74` | Transaction Hash Index Table |
| `b` | 98 | `0x62` | Block Height to Hash Map |

---

### B. Byte-Exact Database Layouts

#### 1. Account State Layout
* **Key Format:**
  * Prefix: `a` (1 byte)
  * Address: `Address` (32 bytes)
  * *Total Key Size:* 33 bytes.
* **Value Format:**
  * Balance: `u64` (8 bytes, Big-Endian)
  * Nonce: `u64` (8 bytes, Big-Endian)
  * Code Hash: `[u8; 32]` (32 bytes)
  * Storage Root: `[u8; 32]` (32 bytes)
  * *Total Value Size:* 80 bytes (fixed size).

#### 2. Contract Code Layout
* **Key Format:**
  * Prefix: `c` (1 byte)
  * Code Hash: `[u8; 32]` (32 bytes)
  * *Total Key Size:* 33 bytes.
* **Value Format:**
  * Bytecode: `[u8]` (Variable length raw EVM bytecode).

#### 3. Contract Storage Layout
* **Key Format:**
  * Prefix: `s` (1 byte)
  * Address: `Address` (32 bytes)
  * Storage Key: `[u8; 32]` (32 bytes)
  * *Total Key Size:* 65 bytes.
* **Value Format:**
  * Storage Value: `[u8; 32]` (32 bytes).

#### 4. Block Header Layout
* **Key Format:**
  * Prefix: `h` (1 byte)
  * Block Hash: `[u8; 32]` (32 bytes)
  * *Total Key Size:* 33 bytes.
* **Value Format:**
  * Serialized Header: Bincode-encoded `BlockHeader` (fixed-width, big-endian format).

#### 5. Block Body Layout
* **Key Format:**
  * Prefix: `d` (1 byte)
  * Block Hash: `[u8; 32]` (32 bytes)
  * *Total Key Size:* 33 bytes.
* **Value Format:**
  * Serialized Transactions: Length-prefixed vector of serialized `TransactionEnvelope` structures.

#### 6. Transaction Hash Index Layout
* **Key Format:**
  * Prefix: `t` (1 byte)
  * Transaction Hash: `[u8; 32]` (32 bytes)
  * *Total Key Size:* 33 bytes.
* **Value Format:**
  * Block Hash: `[u8; 32]` (32 bytes)
  * Transaction Index: `u32` (4 bytes, Big-Endian index in the block).
  * *Total Value Size:* 36 bytes.

#### 7. Block Height Map Layout
* **Key Format:**
  * Prefix: `b` (1 byte)
  * Block Height: `u64` (8 bytes, Big-Endian)
  * *Total Key Size:* 9 bytes.
* **Value Format:**
  * Block Hash: `[u8; 32]` (32 bytes).

---

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

### Determinism Rules:
* Floating-point arithmetic is **strictly forbidden** inside the state transition engine. All balances, fee calculations, and gas multipliers must use fixed-point integer mathematics.
* Any compilation target mismatch or divergent output on a block transition between an ARM64 phone and an x86_64 PC is treated as a critical consensus failure.
