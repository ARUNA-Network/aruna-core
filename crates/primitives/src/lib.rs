//! Base primitives for the ARUNA blockchain.
//! Contains type definitions for Address, Hash, Block, Transaction, Nonce, ChainId, and Difficulty.

pub mod bech32m;
pub mod hash;
pub mod address;
pub mod chain;
pub mod nonce;
pub mod difficulty;
pub mod block;
pub mod transaction;
pub mod errors;
pub mod serialization;

pub use hash::Hash;
pub use address::Address;
pub use chain::ChainId;
pub use nonce::Nonce;
pub use difficulty::Difficulty;
pub use block::{Block, BlockBody, BlockHeader};
pub use transaction::{SignatureType, TransactionEnvelope, TransactionPayload};
pub use errors::PrimitiveError;
pub use serialization::{bincode_options, serialize, deserialize};

use serde::{Deserialize, Serialize};

/// Initial handshake message exchanged between P2P nodes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeMessage {
    /// Current version of the node software.
    pub version: u32,
    /// BLAKE3 hash of the node's public key.
    pub node_id: [u8; 32],
    /// Network stage identifier.
    pub chain_id: ChainId,
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
            state_root: Hash([0xaa; 32]),
            timestamp: 1234567890,
            difficulty: Difficulty(0x12345678),
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
            nonce: Nonce(5),
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
