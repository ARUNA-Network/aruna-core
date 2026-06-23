# Protocol Validation: Block Specification

This document details the block structure, parameters, and serialization layout for the ARUNA Network core protocol.

## 1. Block Size Limits
* **Maximum Block Size Limit:** **2 MB (2,097,152 bytes)**. This includes both the block header and block body (all transaction envelopes and metadata).
* **Rationale:** A 2 MB block size cap ensures that block transmission takes less than 3 seconds on standard mobile/residential connections (reducing partition and uncle block rates) while providing a throughput of approximately 25–40 transactions per second under EVM gas constraints.
* **Size Verification:** Verified during block reception inside `crates/consensus` before any execution occurs. If a block payload exceeds 2 MB, it is dropped immediately without validation.

## 2. Block Fields
A block is split into a **Block Header** (which is solved by miners and signed by stakers) and a **Block Body** (which holds transactions).

### A. Block Header Fields (80 Bytes Binary Target)
The block header is a fixed-size structure used in hashing and mining.
1. **Version (4 bytes / `u32`):** Protocol version identifier.
2. **Previous Block Hash (32 bytes / `[u8; 32]`):** The BLAKE3 hash of the previous block's header.
3. **Transactions Merkle Root (32 bytes / `[u8; 32]`):** The cryptographic root hash of all transactions inside the block body.
4. **Timestamp (8 bytes / `u64`):** POSIX timestamp in seconds (must be greater than the median of the previous 11 blocks).
5. **Difficulty Target (4 bytes / `u32`):** Compact representation of the difficulty target (similar to Bitcoin's nBits).
6. **AHash Nonce (8 bytes / `u64`):** The proof-of-work solution found by community miners running the AHash algorithm.
7. **Validator Signature Root (32 bytes / `[u8; 32]`):** The Merkle root of the public keys and signatures of the stakers who validated this block.
8. **Treasury Root (32 bytes / `[u8; 32]`):** Hash of the protocol treasury allocation transaction outputs.

### B. Block Body Fields
1. **Transactions List:** Array of signed transaction envelopes.
2. **Validator Metadata:** Signatures and public keys validating the block.
3. **Ecosystem Metadata:** Consensus, difficulty tracking, and network indicators.

## 3. Serialization & Encoding
* **Standard:** **Bincode** (binary serialization based on `serde`).
* **Justification:** Bincode has zero overhead, is deterministic, has no padding, and integrates directly with Rust types. JSON or Protobuf is avoided in the core protocol due to parsing overhead and potential non-determinism.
* **Deterministic Rules:** All integers are serialized in big-endian format. Vector lengths are prefix-encoded as `u64`.

## 4. Versioning
* **Opcode/Feature Gates:** Hard forks and soft forks are managed using the 32-bit `Version` field in the block header.
* **Upgrade Logic:** If the block height reaches an upgrade checkpoint, the consensus engine validates block headers according to the new rules enforced by that version. Nodes that do not upgrade will reject blocks with the new version and fork off.

## 5. Block Hashing
* **Algorithm:** **BLAKE3**.
* **Formula:**
  `BlockHash = BLAKE3(Serialized Block Header)`
* **Rationale:** BLAKE3 is selected because it is secure, parallelizable, and significantly faster than SHA-256 or Keccak-256 on both x86_64 and ARM64 CPUs.

## 6. Merkle Tree
* **Type:** Binary Merkle Tree.
* **Leaves:** Each leaf is the BLAKE3 hash of a serialized transaction:
  `Leaf_i = BLAKE3(Serialized_Transaction_i)`
* **Parent Nodes:** Calculated by concatenating and hashing child pairs:
  `Parent = BLAKE3(Left_Child || Right_Child)`
* **Odd Leaves:** If the number of transactions is odd, the last leaf hash is duplicated to form a pair.
* **Empty Block:** If a block contains no transactions, the `Merkle Root` is set to `[0u8; 32]`.
