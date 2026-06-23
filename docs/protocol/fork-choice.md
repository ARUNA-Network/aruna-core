# Protocol Specification: Fork Choice Rule & Chain Reorganizations

This document defines the rules for resolving competing chain forks, calculating cumulative difficulty work, and enforcing the immutable checkpoint reorganization limits.

## 1. Fork Choice Rule (FCR) Priorities
When a node receives competing blocks from different peers, it resolves the fork by applying FCR rules in the following order:

1. **Validation Gate:**
   * Any block header containing invalid signatures, invalid AHash PoW solutions, timestamps out of range, or invalid transactions is rejected immediately.
2. **Highest Accumulated Work (Most Work Chain):**
   * The node prefers the chain with the highest cumulative Proof of Work difficulty.
   * **Cumulative Difficulty Calculation:**
     $$\text{CumulativeDifficulty} = \sum_{i=\text{fork\_height}}^{\text{tip}} 2^{\text{Difficulty}_i}$$
     * Where $\text{Difficulty}_i$ is the target difficulty threshold of block $i$.
3. **Highest Finalized Stake Weight:**
   * If two chains have identical cumulative work (e.g. during a short-term split), the node chooses the chain that contains the block signed by the highest cumulative PoS stake weight at the last checkpoint.
4. **Earliest Network Acceptance:**
   * If both work and stake weights are tied, the node prefers the block it received first through its libp2p network connection.

---

## 2. Reorganization Limits (Checkpoint Immutability)
* **Finality Epoch:** An epoch is defined as **2,880 blocks** (~24 hours).
* **Checkpoint Finality:** At the end of each epoch, validators execute a checkpoint vote. Once a checkpoint block reaches >66.7% validator signatures, it is finalized.
* **Reorg Limit Rule:** A node is **strictly prohibited** from performing a chain reorganization that reverts or modifies blocks below the last finalized checkpoint height.
* **Security Outcome:** This mitigates long-range attacks. Even if an attacker rents 100x hashrate and attempts to rewrite the chain history from 10,000 blocks ago, all nodes will drop the attacker's fork because it attempts to overwrite a finalized checkpoint.

---

## 3. Database Reorganization & Rollback Pipeline
When a valid fork with higher cumulative difficulty is accepted:
1. **Identify Fork Point:** Find the common ancestor block where the current chain and the new fork diverged.
2. **Rollback State:** Disconnect blocks from the current tip down to the fork point. Roll back all balance transfers, nonces, and smart contract storage mutations in RocksDB.
3. **Apply Fork State:** Execute and apply the transactions of the new fork blocks sequentially from the fork point to the new tip.
4. **Update Tip:** Update the RocksDB canonical chain tip reference to the hash of the new tip block.
