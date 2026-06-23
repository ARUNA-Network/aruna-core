//! Bech32m (BIP-350) encoding and decoding implementation for ARUNA addresses.

use thiserror::Error;

const CHARSET: &[char] = &[
    'q', 'p', 'z', 'r', 'y', '9', 'x', '8',
    'g', 'f', '2', 't', 'v', 'd', 'w', '0',
    's', '3', 'j', 'n', '5', '4', 'k', 'h',
    'c', 'e', '6', 'm', 'u', 'a', '7', 'l',
];

const BECH32M_CONST: u32 = 0x2bc830f3;
const GENERATOR: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Bech32mError {
    #[error("Invalid character in address: {0}")]
    InvalidChar(char),
    #[error("Invalid length")]
    InvalidLength,
    #[error("Separator '1' not found")]
    NoSeparator,
    #[error("HRP is empty")]
    EmptyHrp,
    #[error("Invalid human-readable part (HRP): {0}")]
    InvalidHrp(String),
    #[error("Invalid checksum")]
    InvalidChecksum,
    #[error("Invalid padding or data alignment")]
    InvalidPadding,
    #[error("Data conversion error: {0}")]
    DataError(String),
}

/// Verify if the given HRP prefix is a valid ARUNA prefix.
pub fn is_valid_hrp(hrp: &str) -> bool {
    matches!(
        hrp,
        "sum" | "sumc" | "kal" | "kalc" | "sul" | "sulc" | "pap" | "papc" | "jaw" | "jawc"
    )
}

fn polymod(values: &[u8]) -> u32 {
    let mut chk = 1u32;
    for &v in values {
        let b = chk >> 25;
        chk = ((chk & 0x1ffffff) << 5) ^ (v as u32);
        for i in 0..5 {
            if ((b >> i) & 1) != 0 {
                chk ^= GENERATOR[i];
            }
        }
    }
    chk
}

fn hrp_expand(hrp: &str) -> Vec<u8> {
    let mut ret = Vec::with_capacity(hrp.len() * 2 + 1);
    for c in hrp.bytes() {
        ret.push(c >> 5);
    }
    ret.push(0);
    for c in hrp.bytes() {
        ret.push(c & 31);
    }
    ret
}

fn verify_checksum(hrp: &str, data: &[u8]) -> bool {
    let mut combined = hrp_expand(hrp);
    combined.extend_from_slice(data);
    polymod(&combined) == BECH32M_CONST
}

fn create_checksum(hrp: &str, data: &[u8]) -> Vec<u8> {
    let mut combined = hrp_expand(hrp);
    combined.extend_from_slice(data);
    combined.extend_from_slice(&[0; 6]);
    let mod_val = polymod(&combined) ^ BECH32M_CONST;
    let mut ret = Vec::with_capacity(6);
    for i in 0..6 {
        ret.push(((mod_val >> (5 * (5 - i))) & 31) as u8);
    }
    ret
}

/// Convert bits from one base size to another (e.g. 8-bit bytes to 5-bit values).
pub fn convert_bits(data: &[u8], from_bits: u32, to_bits: u32, pad: bool) -> Result<Vec<u8>, Bech32mError> {
    let mut acc = 0u32;
    let mut bits = 0u32;
    let mut ret = Vec::new();
    let maxv = (1u32 << to_bits) - 1;
    let max_acc = (1u32 << (from_bits + to_bits - 1)) - 1;
    for &value in data {
        acc = ((acc << from_bits) | (value as u32)) & max_acc;
        bits += from_bits;
        while bits >= to_bits {
            bits -= to_bits;
            ret.push(((acc >> bits) & maxv) as u8);
        }
    }
    if pad {
        if bits > 0 {
            ret.push(((acc << (to_bits - bits)) & maxv) as u8);
        }
    } else if bits >= from_bits || ((acc << (to_bits - bits)) & maxv) != 0 {
        return Err(Bech32mError::InvalidPadding);
    }
    Ok(ret)
}

/// Encodes binary data to a Bech32m string.
pub fn encode(hrp: &str, data: &[u8]) -> Result<String, Bech32mError> {
    if hrp.is_empty() {
        return Err(Bech32mError::EmptyHrp);
    }
    if !is_valid_hrp(hrp) {
        return Err(Bech32mError::InvalidHrp(hrp.to_string()));
    }
    let converted = convert_bits(data, 8, 5, true)?;
    let checksum = create_checksum(hrp, &converted);
    let mut combined = converted;
    combined.extend_from_slice(&checksum);

    let mut ret = String::with_capacity(hrp.len() + 1 + combined.len());
    ret.push_str(hrp);
    ret.push('1');
    for &index in &combined {
        if index >= 32 {
            return Err(Bech32mError::DataError("Value out of base32 range".to_string()));
        }
        ret.push(CHARSET[index as usize]);
    }
    Ok(ret)
}

/// Decodes a Bech32m string back to the human-readable part and raw bytes.
pub fn decode(s: &str) -> Result<(String, Vec<u8>), Bech32mError> {
    if s.len() < 8 || s.len() > 90 {
        return Err(Bech32mError::InvalidLength);
    }
    if s.chars().any(|c| c.is_uppercase()) && s.chars().any(|c| c.is_lowercase()) {
        return Err(Bech32mError::InvalidChar('A')); // Mixed case error
    }
    let lowercase_str = s.to_lowercase();
    let pos = lowercase_str.rfind('1').ok_or(Bech32mError::NoSeparator)?;
    if pos == 0 {
        return Err(Bech32mError::EmptyHrp);
    }
    let hrp = &lowercase_str[..pos];
    if !is_valid_hrp(hrp) {
        return Err(Bech32mError::InvalidHrp(hrp.to_string()));
    }
    let data_chars = &lowercase_str[pos + 1..];
    if data_chars.len() < 6 {
        return Err(Bech32mError::InvalidLength);
    }

    let mut data = Vec::with_capacity(data_chars.len());
    for c in data_chars.chars() {
        let index = CHARSET
            .iter()
            .position(|&x| x == c)
            .ok_or(Bech32mError::InvalidChar(c))?;
        data.push(index as u8);
    }

    if !verify_checksum(hrp, &data) {
        return Err(Bech32mError::InvalidChecksum);
    }

    let payload = &data[..data.len() - 6];
    let decoded = convert_bits(payload, 5, 8, false)?;
    Ok((hrp.to_string(), decoded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_bech32m_roundtrip() {
        let hrp = "jaw";
        let data = [0xab; 20];
        let encoded = encode(hrp, &data).unwrap();
        assert!(encoded.starts_with("jaw1"));
        
        let (decoded_hrp, decoded_data) = decode(&encoded).unwrap();
        assert_eq!(decoded_hrp, hrp);
        assert_eq!(decoded_data, data);
    }

    #[test]
    fn test_invalid_hrp() {
        let data = [0x00; 20];
        let result = encode("invalid", &data);
        assert!(matches!(result, Err(Bech32mError::InvalidHrp(_))));
    }

    #[test]
    fn test_corrupted_checksum() {
        let hrp = "jaw";
        let data = [0x12; 20];
        let mut encoded = encode(hrp, &data).unwrap();
        // Corrupt the last character
        encoded.pop();
        encoded.push('q'); // change last char
        
        let decode_result = decode(&encoded);
        assert!(decode_result.is_err());
    }

    #[test]
    fn test_contract_prefix() {
        let hrp = "jawc";
        let data = [0xcd; 20];
        let encoded = encode(hrp, &data).unwrap();
        assert!(encoded.starts_with("jawc1"));
        
        let (decoded_hrp, decoded_data) = decode(&encoded).unwrap();
        assert_eq!(decoded_hrp, hrp);
        assert_eq!(decoded_data, data);
    }
}
