//! Cryptographic signatures and hashing primitives for ARUNA.
//!
//! Currently Implemented:
//! - BLAKE3 (hashing)
//! - Ed25519 (signatures)
//!
//! Planned:
//! - Argon2 (mining memory expansion)
//! - AES (mining mixing stage)
//! - secp256k1 (EVM compatibility)

use aruna_primitives::{Hash, BlockHeader, serialize};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Ed25519 signature error: {0}")]
    Signature(#[from] ed25519_dalek::SignatureError),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid key or signature length")]
    InvalidLength,
}

/// Compute a BLAKE3 hash for the given byte slice.
pub fn blake3_hash(data: &[u8]) -> Hash {
    let digest = blake3::hash(data);
    Hash(*digest.as_bytes())
}

/// Compute a BLAKE3 hash of a serialized block header.
pub fn hash_block_header(header: &BlockHeader) -> Result<Hash, CryptoError> {
    let bytes = serialize(header).map_err(|e| CryptoError::Serialization(e.to_string()))?;
    Ok(blake3_hash(&bytes))
}

/// Wrapper around Ed25519 signing credentials.
pub struct Ed25519Keypair {
    signing_key: SigningKey,
}

impl Ed25519Keypair {
    /// Create a keypair from a 32-byte private seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        Self { signing_key }
    }

    /// Generate a new random keypair.
    pub fn generate() -> Self {
        use rand_core::OsRng;
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Get the public key bytes.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Sign a message payload, returning a 64-byte signature.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature = self.signing_key.sign(message);
        signature.to_bytes()
    }

    /// Export the 32-byte seed (private scalar). Required for wallet backup.
    /// The seed is the canonical private key material for this keypair.
    /// Store securely — anyone with this value controls the associated funds.
    pub fn seed_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

/// Verifies Ed25519 signatures.
pub struct Ed25519Verifier;

impl Ed25519Verifier {
    /// Verify an Ed25519 signature against a public key and raw message.
    pub fn verify(pubkey: &[u8; 32], message: &[u8], signature_bytes: &[u8; 64]) -> Result<(), CryptoError> {
        let verifying_key = VerifyingKey::from_bytes(pubkey)?;
        let signature = Signature::from_bytes(signature_bytes);
        verifying_key.verify(message, &signature)?;
        Ok(())
    }
}

/// Derives a 20-byte public key hash from a raw public key.
/// Pipeline: PubKeyHash = BLAKE3(PublicKey)[0..20]
pub fn derive_pubkey_hash(public_key: &[u8]) -> [u8; 20] {
    let hash = blake3::hash(public_key);
    let mut out = [0u8; 20];
    out.copy_from_slice(&hash.as_bytes()[0..20]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use aruna_primitives::Difficulty;

    #[test]
    fn test_blake3_hashing() {
        let data = b"hello aruna";
        let hash = blake3_hash(data);
        assert_ne!(hash, Hash::zero());
        
        let expected = blake3::hash(data);
        assert_eq!(hash.0, *expected.as_bytes());
    }

    #[test]
    fn test_hash_block_header() {
        let header = BlockHeader {
            version: 1,
            prev_block_hash: Hash([0; 32]),
            merkle_root: Hash([0; 32]),
            state_root: Hash([0; 32]),
            timestamp: 1234567,
            difficulty: Difficulty(100),
            nonce: 42,
            validator_root: Hash([0; 32]),
            treasury_root: Hash([0; 32]),
        };
        let hash = hash_block_header(&header).unwrap();
        assert_ne!(hash, Hash::zero());
    }

    #[test]
    fn test_ed25519_signing_and_verification() {
        let keypair = Ed25519Keypair::generate();
        let pubkey = keypair.public_key_bytes();
        let message = b"consensus vote block 100";
        
        let signature = keypair.sign(message);
        
        // Verify successfully
        let result = Ed25519Verifier::verify(&pubkey, message, &signature);
        assert!(result.is_ok());
        
        // Verify fails on altered message
        let invalid_message = b"consensus vote block 101";
        let result_invalid = Ed25519Verifier::verify(&pubkey, invalid_message, &signature);
        assert!(result_invalid.is_err());
    }

    #[test]
    fn test_address_derivation() {
        let pubkey = [0u8; 32];
        let pkh = derive_pubkey_hash(&pubkey);
        assert_eq!(pkh.len(), 20);
        assert_ne!(pkh, [0u8; 20]);
    }
}
