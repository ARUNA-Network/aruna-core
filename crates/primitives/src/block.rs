use serde::{Deserialize, Serialize};
use crate::{Hash, Difficulty, TransactionEnvelope};

/// Fixed-size block header containing metadata and Merkle roots.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Protocol version identifier.
    pub version: u32,
    /// BLAKE3 hash of the previous block's header.
    pub prev_block_hash: Hash,
    /// Cryptographic Merkle root hash of all transactions inside the block body.
    pub merkle_root: Hash,
    /// POSIX timestamp in seconds.
    pub timestamp: u64,
    /// Compact difficulty target.
    pub difficulty: Difficulty,
    /// Proof-of-work nonce found by miners running the AHash algorithm.
    pub nonce: u64,
    /// Merkle root of validator public keys and signatures.
    pub validator_root: Hash,
    /// Hash of the protocol treasury allocation outputs.
    pub treasury_root: Hash,
}

/// The body of a block holding list of transaction envelopes and consensus metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockBody {
    /// Array of signed transaction envelopes.
    pub transactions: Vec<TransactionEnvelope>,
    /// Staker signatures and public keys validating this block.
    pub validator_metadata: Vec<u8>,
    /// Consensus, difficulty, and network indicators.
    pub ecosystem_metadata: Vec<u8>,
}

/// A block composed of a BlockHeader and BlockBody.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// Block header metadata.
    pub header: BlockHeader,
    /// Block transactions and validator metadata.
    pub body: BlockBody,
}
