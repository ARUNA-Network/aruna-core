use serde::{Deserialize, Serialize};
use std::fmt;
use crate::{bech32m, PrimitiveError};

/// A 32-byte account address.
/// Internally wraps a 32-byte array. Standard 20-byte public key hashes (BLAKE3(PubKey)[0..20])
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
