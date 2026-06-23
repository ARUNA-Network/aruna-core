use bincode::Options;
use serde::{Deserialize, Serialize};

/// Retrieve standard Bincode options matching ARUNA Network's serialization rules
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
