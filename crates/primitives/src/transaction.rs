use serde::{Deserialize, Serialize};
use crate::{Address, Nonce};

/// Supported signature schemes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SignatureType {
    /// Standard wallets (Ed25519)
    Ed25519 = 0,
    /// EVM wallets (secp256k1)
    Secp256k1 = 1,
}

/// The core content (payload) of an ARUNA transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionPayload {
    /// Sequence number representing the total number of transactions sent from the sender's account.
    pub nonce: Nonce,
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

/// Signed transaction envelope enclosing payload and signature data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionEnvelope {
    /// Serialized transaction payload.
    pub payload: TransactionPayload,
    /// Cryptographic signature type used to sign the payload.
    pub signature_type: SignatureType,
    /// Cryptographic signature bytes (64 bytes for Ed25519, 65 bytes for secp256k1).
    pub signature: Vec<u8>,
    /// Cryptographic public key of the sender.
    pub public_key: Vec<u8>,
}
