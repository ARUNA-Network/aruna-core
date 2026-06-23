# RFC-0005: Consensus Finality & Checkpoint Agreement

## Status
Accepted

## 1. Background
This RFC outlines the finality checkpoint model, validator consensus votes, and chain reorganization limits.

## 2. Proposed Specification

### A. Finality Epochs
* **Epoch Length:** **2,880 blocks** (~24 hours at 30s block time).
* **Checkpoint Trigger:** The block at the end of each epoch (e.g. block 2,880, 5,760, etc.) is flagged as an epoch boundary block.

### B. Validator Voting & Checkpoint Agreement
1. At the epoch boundary block, active validators sign the block hash.
2. Signatures are gossiped across the peer-to-peer network.
3. Once a block accumulates signatures representing **>66.7%** of the total staked weight, the block hash is committed to RocksDB as a **Finalized Checkpoint**.

### C. Reorganization Isolation Rule
* Nodes are strictly prohibited from reorganizing the chain to any fork that branches below the height of the last finalized checkpoint.
* This eliminates long-range attack vectors where old validator keys are compromised to sign a competing chain history.
