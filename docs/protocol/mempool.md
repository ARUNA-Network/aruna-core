# Protocol Specification: Mempool Policy

This document defines the transaction validation rules, replacement limits, queue prioritization, and eviction policies for the node's local transaction mempool.

## 1. Mempool Entrance Validation
Before a transaction envelope is accepted into a node's local mempool, it must pass the following checks:
1. **Signature Verification:** The signature must correspond to the sender's public key (using Ed25519 or secp256k1).
2. **Nonce Verification:** The transaction nonce `Tx.nonce` must be greater than or equal to the sender's on-chain nonce:
   `Tx.nonce >= Account.nonce`
3. **Balance Verification:** The sender must have a balance equal to or greater than the transfer amount plus the maximum fee:
   `Account.balance >= Tx.amount + Tx.max_fee`
4. **Fee Verification:** The fee offered must be equal to or greater than the minimum fee floor:
   `Tx.fee >= BaseFeePerByte * Tx.size`

## 2. Queue Prioritization
* **Sorting Metrics:** Pending transactions are sorted inside the mempool using a two-tier priority model:
  1. **Gas Price (for EVM transactions):** Transactions offering higher gas prices are prioritized.
  2. **Fee Density (for standard transfers):** Transactions offering higher micro-ARU per byte are prioritized.
* **Nonce Sequencing:** For a single account, transactions must be executed sequentially. A transaction with nonce `N+1` cannot be included in a block before transaction `N`.

## 3. Transaction Replacement (Replace-By-Fee)
A user can replace a pending transaction in the mempool with a new transaction (e.g. to speed up execution) under these rules:
* **Identification:** The new transaction must have the same `sender` and `nonce` as the pending transaction.
* **Replacement Threshold:** The new transaction must increase the fee or gas price by **at least 10%** compared to the original transaction.
* **Action:** Once verified, the old transaction is evicted from the mempool and replaced by the new transaction.

## 4. Size Limits & Eviction Policy
* **Mempool Size Cap:** Default maximum size is **50,000 transactions**.
* **Eviction Trigger:** When the mempool reaches its 50k limit and a new valid transaction arrives:
  1. The mempool identifies transactions in the lowest 10% fee percentile.
  2. If the new transaction offers a higher fee than the lowest transaction, the lowest transaction is evicted.
  3. If the new transaction has a lower fee than the lowest transaction, the new transaction is rejected.

## 5. Expiration & Double Spend Cleanups
* **Mempool Expiration:** Transactions remaining in the mempool for more than **72 hours** (8,640 blocks) are automatically expired and evicted.
* **Block Commit Cleanup:** When a new block is committed to the canonical chain, the mempool:
  1. Evicts all transactions included in that block.
  2. Evicts all transactions from the same senders that have nonces lower than or equal to the newly updated account nonces.
