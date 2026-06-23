use serde::{Deserialize, Serialize};

/// Represents the compact representation of mining difficulty target (nBits).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Difficulty(pub u32);

impl From<u32> for Difficulty {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl From<Difficulty> for u32 {
    fn from(diff: Difficulty) -> Self {
        diff.0
    }
}
