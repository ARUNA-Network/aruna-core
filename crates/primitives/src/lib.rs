//! Base primitives for the ARUNA blockchain.
//! Contains type definitions for Address, Hash, Block, Transaction, and Nonce.

pub mod bech32m;

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Error type for ARUNA primitive conversions.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveError {
    #[error("Invalid hex string: {0}")]
    InvalidHex(#[from] hex::FromHexError),
    #[error("Invalid slice length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },
    #[error("Bech32m error: {0}")]
    Bech32m(#[from] bech32m::Bech32mError),
    #[error("Address format error: {0}")]
    AddressFormat(String),
}

/// A 32-byte cryptographic hash (BLAKE3 digest in ARUNA).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// Create a zero hash.
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Create from a 32-byte array.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Try to create from a byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self, PrimitiveError> {
        if slice.len() != 32 {
            return Err(PrimitiveError::InvalidLength {
                expected: 32,
                got: slice.len(),
            });
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    /// Try to parse from a hex string.
    pub fn from_hex(hex_str: &str) -> Result<Self, PrimitiveError> {
        let decoded = hex::decode(hex_str.trim_start_matches("0x"))?;
        Self::from_slice(&decoded)
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(0x{})", hex::encode(self.0))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// A 32-byte account address.
/// Internally wraps a 32-byte array. Standard 20-byte public key hashes (RIPEMD160(SHA256(PubKey)))
/// are left-padded with zeros (stored in the last 20 bytes: 12..32) for EVM compatibility.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Address(pub [u8; 32]);

impl Address {
    /// Create a zero address.
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Create an Address wrapping raw 32 bytes.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create an Address from a 20-byte public key hash (left-padded with zeros).
    pub fn from_pubkey_hash(hash: [u8; 20]) -> Self {
        let mut bytes = [0u8; 32];
        bytes[12..32].copy_from_slice(&hash);
        Self(bytes)
    }

    /// Extract the 20-byte public key hash from the address (bytes 12..32).
    pub fn pubkey_hash(&self) -> [u8; 20] {
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&self.0[12..32]);
        hash
    }

    /// Convert address to a Bech32m encoded string with a specific network prefix (HRP).
    pub fn to_bech32m(&self, hrp: &str) -> Result<String, PrimitiveError> {
        let pkh = self.pubkey_hash();
        bech32m::encode(hrp, &pkh).map_err(PrimitiveError::from)
    }

    /// Decode and create an Address from a Bech32m string.
    /// Returns the parsed human-readable part (HRP) and the derived Address.
    pub fn from_bech32m(s: &str) -> Result<(String, Self), PrimitiveError> {
        let (hrp, pkh) = bech32m::decode(s)?;
        if pkh.len() != 20 {
            return Err(PrimitiveError::AddressFormat(format!(
                "Decoded pubkey hash length must be 20 bytes, got {}",
                pkh.len()
            )));
        }
        let mut hash_bytes = [0u8; 20];
        hash_bytes.copy_from_slice(&pkh);
        Ok((hrp, Self::from_pubkey_hash(hash_bytes)))
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address(0x{})", hex::encode(self.0))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

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
    pub difficulty: u32,
    /// Proof-of-work nonce found by miners running the AHash algorithm.
    pub nonce: u64,
    /// Merkle root of validator public keys and signatures.
    pub validator_root: Hash,
    /// Hash of the protocol treasury allocation outputs.
    pub treasury_root: Hash,
}

/// The core content (payload) of an ARUNA transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionPayload {
    /// Nonce representing the total number of transactions sent from the sender's account.
    pub nonce: u64,
    /// Bech32m-decoded address of the sender.
    pub sender: Address,
    /// Bech32m-decoded address of the recipient.
    pub recipient: Address,
    /// Value of ARU coins transferred (in micro-ARU: 1 ARU = 1,000,000 micro-ARU).
    pub amount: u64,
    /// Transaction fee offered to miners and validators (in micro-ARU).
    pub fee: u64,
    /// Maximum EVM gas allowed for contract deployment or execution.
    pub gas_limit: u64,
    /// Fee offered per unit of gas.
    pub gas_price: u64,
    /// Optional variable-length bytes (bytecode or execution arguments).
    pub data: Vec<u8>,
}

/// Supported signature schemes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SignatureType {
    /// Standard wallets (Ed25519)
    Ed25519 = 0,
    /// EVM wallets (secp256k1)
    Secp256k1 = 1,
}

/// Signed transaction envelope enclosing payload and signature data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionEnvelope {
    /// Serialized transaction payload.
    pub payload: TransactionPayload,
    /// Cryptographic signature type used to sign the payload.
    pub signature_type: SignatureType,
    /// Cryptographic signature bytes (64 bytes for Ed25519, 65 bytes for secp256k1).
    pub signature: Vec<u8>,
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

/// Initial handshake message exchanged between P2P nodes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeMessage {
    /// Current version of the node software.
    pub version: u32,
    /// BLAKE3 hash of the node's public key.
    pub node_id: [u8; 32],
    /// Network stage identifier.
    pub chain_id: u32,
    /// The height of the canonical tip of the sending node.
    pub current_height: u64,
    /// Capability flags (Full Node, Validator, Archive, etc.)
    pub capabilities: u8,
}

/// P2P block history synchronization request stream payload.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncRequestMessage {
    /// Starting block height for retrieval.
    pub start_height: u64,
    /// End block height for retrieval.
    pub end_height: u64,
    /// Maximum blocks allowed in a single response.
    pub block_limit: u16,
}

/// P2P block history synchronization response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncResponseMessage {
    /// Response status (0 = Success, 1 = Out of Range, 2 = Internal Error).
    pub status: u8,
    /// List of serialized blocks.
    pub blocks: Vec<Block>,
}

/// Retrive standard Bincode options matching ARUNA Network's serialization rules
/// (Big Endian byte order and fixed-width integer encoding).
pub fn bincode_options() -> impl bincode::Options {
    use bincode::Options;
    bincode::options()
        .with_big_endian()
        .with_fixint_encoding()
}

/// Serialize a type into big-endian fixed-integer Bincode bytes.
pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, bincode::Error> {
    bincode_options().serialize(value)
}

/// Deserialize big-endian fixed-integer Bincode bytes back into a Rust type.
pub fn deserialize<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, bincode::Error> {
    bincode_options().deserialize(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_padding_evm_compatible() {
        let pkh = [0xde; 20];
        let address = Address::from_pubkey_hash(pkh);
        
        // Ensure left-padded: bytes 0..12 are 0
        assert_eq!(&address.0[0..12], &[0u8; 12]);
        // Bytes 12..32 contain the hash
        assert_eq!(&address.0[12..32], &pkh);
        // pubkey_hash method extracts it back
        assert_eq!(address.pubkey_hash(), pkh);
    }

    #[test]
    fn test_address_bech32m_roundtrip() {
        let pkh = [0x55; 20];
        let address = Address::from_pubkey_hash(pkh);
        let hrp = "jaw";
        
        let encoded = address.to_bech32m(hrp).unwrap();
        assert!(encoded.starts_with("jaw1"));
        
        let (decoded_hrp, decoded_address) = Address::from_bech32m(&encoded).unwrap();
        assert_eq!(decoded_hrp, hrp);
        assert_eq!(decoded_address, address);
    }

    #[test]
    fn test_bincode_serialization_endianness() {
        let header = BlockHeader {
            version: 1,
            prev_block_hash: Hash([0x11; 32]),
            merkle_root: Hash([0x22; 32]),
            timestamp: 1234567890,
            difficulty: 0x12345678,
            nonce: 9876543210,
            validator_root: Hash([0x33; 32]),
            treasury_root: Hash([0x44; 32]),
        };
        
        let bytes = serialize(&header).unwrap();
        
        // Verify big-endian representation of Version (1) -> [0, 0, 0, 1]
        assert_eq!(&bytes[0..4], &[0, 0, 0, 1]);
        
        // Verify roundtrip
        let decoded: BlockHeader = deserialize(&bytes).unwrap();
        assert_eq!(decoded, header);
    }

    #[test]
    fn test_transaction_payload_serialization() {
        let tx = TransactionPayload {
            nonce: 5,
            sender: Address::from_pubkey_hash([0xaa; 20]),
            recipient: Address::from_pubkey_hash([0xbb; 20]),
            amount: 1000000,
            fee: 10,
            gas_limit: 21000,
            gas_price: 100,
            data: vec![1, 2, 3, 4],
        };
        
        let bytes = serialize(&tx).unwrap();
        let decoded: TransactionPayload = deserialize(&bytes).unwrap();
        assert_eq!(decoded, tx);
    }
}
