use serde::{Deserialize, Serialize};

/// Identifies the specific ARUNA network stage (Sumatera, Jawa, etc.).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChainId(pub u32);

impl ChainId {
    /// Sumatera Testnet Chain ID (1).
    pub const SUMATERA: Self = Self(1);
    
    /// Kalimantan Testnet Chain ID (2).
    pub const KALIMANTAN: Self = Self(2);
    
    /// Sulawesi Testnet Chain ID (3).
    pub const SULAWESI: Self = Self(3);
    
    /// Papua Release Candidate Chain ID (4).
    pub const PAPUA: Self = Self(4);
    
    /// Jawa Mainnet Chain ID (7777).
    pub const JAWA: Self = Self(7777);
}

impl From<u32> for ChainId {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl From<ChainId> for u32 {
    fn from(id: ChainId) -> Self {
        id.0
    }
}
