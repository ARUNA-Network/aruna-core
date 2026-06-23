# ADR-0016: State Root Strategy

## Status
Proposed

## Context
In account-based blockchains, the global ledger state (balances, nonces, code, contract storage) changes with every block. Validating a block requires not only validating transaction executions but also confirming that the resulting database state is identical across all nodes. 

To achieve this:
1. The block header must contain a `state_root` field.
2. The consensus engine must calculate this root after executing all transactions and compare it to the block header's root.
3. Light clients must be able to verify proofs of account balances or storage values against this root without downloading the entire state database.

## Problem
In EVM ecosystems, state is structured as a Merkle Patricia Trie (MPT) over a key-value database. While MPT provides cryptographic integrity and supports Merkle proofs, it is complex to implement, has significant CPU overhead during recalculation, and can lead to disk space bloat due to intermediate nodes storage.

For ARUNA, which natively targets low-power CPUs, ARM servers, and Android phones, we need a state root strategy that:
1. Is computationally lightweight and fast to calculate.
2. Supports deterministic, reproducible validation across architectures.
3. Enables light client validation with minimal data transfer.

## Decision
We propose utilizing a **BLAKE3 Merkle Mountain Range (MMR) or a rolling state hash accumulator** over the active account state database.

### 1. Rolling Hash Accumulator Strategy (Phase 1 / Sumatera)
To avoid the overhead of a full Merkle Patricia Trie during early testnets, the state root is computed as a rolling hash accumulator of all modified account keys and values in the block, sorted lexicographically by address:
$$\text{StateRoot}_{N} = \text{BLAKE3}(\text{StateRoot}_{N-1} \parallel \text{BlockUpdatesHash})$$
Where `BlockUpdatesHash` is the BLAKE3 root of the lexicographically sorted array of serialized `(Address, AccountState)` changes applied in block `N`.

### 2. Merkle Mountain Range (MMR) Transition (Phase 2)
For mainnet scalability, we will implement a BLAKE3 Merkle Mountain Range (MMR) index. Each account leaf is inserted into an MMR tree. Since RocksDB keys are sorted, we can build a deterministic tree structure that allows nodes to query and verify state integrity with logarithmic proof sizes, which is highly friendly to background mobile clients.

## Consequences
* **Positive:**
  * **Low CPU/Memory Usage:** BLAKE3 MMR calculation is significantly faster than standard Ethereum Keccak-256 Merkle Patricia Trie updates.
  * **ARM Friendliness:** Low power consumption during verification loops on background Android devices.
* **Negative:**
  * Requires custom proof verification code in light clients, as standard Ethereum tooling expects MPT layouts.
