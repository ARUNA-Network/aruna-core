# Protocol Validation: Consensus Specification

This document details the consensus rules, finality conditions, difficulty adjustment formulas, and hybrid validation constraints of the ARUNA Network.

## 1. Hybrid Consensus Parameters
ARUNA implements a hybrid consensus engine utilizing both Proof of Work (PoW) hashrate and Proof of Stake (PoS) validator signatures:
* **70% PoW Influence:** Blocks must solve the AHash v1 difficulty target.
* **25% PoS Influence:** Blocks must receive verification signatures from active validators representing a supermajority of the total staked weight.
* **5% Treasury Influence:** Protocol mints 5% of block rewards to the Treasury automatically.

### Block Verification Checklist:
For a block to be appended to the local chain, it must satisfy:
1. **AHash Proof:** `AHash(Block Header) <= DifficultyTarget`.
2. **Validator Signatures:** Contains signatures from validators representing **>66.7%** of the active validator stake.
3. **Transition Validation:** All transactions in the block must be valid and execute deterministically.

## 2. Difficulty Adjustment Algorithm
To maintain a stable **30-second block time**, the difficulty target is adjusted on every block using a **Weighted Moving Average (WMA)** algorithm:

### Difficulty Adjustment Formula:
* **Window Size:** Last 120 blocks.
* **Calculation:**
  $$\text{Target}_{N+1} = \text{Target}_N \times \frac{\text{ActualTime}}{\text{ExpectedTime}}$$
  * Where $\text{ExpectedTime} = 120 \text{ blocks} \times 30 \text{ seconds} = 3,600 \text{ seconds}$.
  * $\text{ActualTime}$ is the timestamp difference between Block $N$ and Block $N-120$.
* **Damping Factor:** To prevent difficulty oscillations and hashing spikes (difficulty shocks), the target adjustment is capped at a maximum of $\pm 25\%$ change per block.

## 3. Finality Model & Checkpoints
ARUNA uses a hybrid finality model to prevent long-range reorganization attacks:
* **Soft Finality (Confirmations):** 4 blocks (~2 minutes). Standard transactions are considered safe from minor reorgs after 4 block confirmations.
* **Hard Finality (Validator Checkpoints):** Every epoch (defined as **2,880 blocks** or approximately 24 hours), active validators perform a checkpoint agreement vote.
  * Once a block height is signed and finalized by a supermajority (>66.7% of stake), it is written to the RocksDB storage as a **Finalized Checkpoint**.
  * **Immutability Rule:** No node will accept a chain reorganization that attempts to revert or modify blocks prior to the last finalized checkpoint, mathematically securing the history from long-range rewrite attacks.
