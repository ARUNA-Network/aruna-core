# RFC-0008: Mempool Admission & Eviction Policies

## Status
Accepted

## 1. Background
The local node mempool stores pending transactions. This RFC defines the mempool capacity limits, prioritization algorithms, replace-by-fee rules, and transaction expiration bounds.

## 2. Proposed Specification

### A. Size Limits & Eviction
* **Max Capacity:** **50,000 transactions**.
* **Eviction rule:** If the mempool is full and a new valid transaction arrives, the node evaluates the lowest 10% fee percentile. If the new transaction offers a higher fee, the lowest transaction is evicted.

### B. Replace-By-Fee (RBF)
* **Identification:** New transaction has the same `sender` and `nonce` as a pending transaction.
* **Price Step:** The new transaction must increase the fee or gas price by **at least 10%** to overwrite the old transaction.

### C. Expiration
* **Timeout:** Transactions remaining in the mempool for more than **72 hours** (8,640 blocks) are evicted.
