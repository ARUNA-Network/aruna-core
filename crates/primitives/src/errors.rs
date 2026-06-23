use thiserror::Error;
use crate::bech32m;

/// Error type for ARUNA primitive conversions.
#[derive(Error, Debug, Clone, PartialEq)]
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
