use serde::{Deserialize, Serialize};
use std::fmt;
use crate::PrimitiveError;

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
