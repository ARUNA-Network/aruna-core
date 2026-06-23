use serde::{Deserialize, Serialize};

/// Represents the transaction nonce (sequence number) for an account.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nonce(pub u64);

impl Nonce {
    /// Zero nonce.
    pub fn zero() -> Self {
        Self(0)
    }

    /// Increment the nonce.
    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u64> for Nonce {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<Nonce> for u64 {
    fn from(nonce: Nonce) -> Self {
        nonce.0
    }
}
