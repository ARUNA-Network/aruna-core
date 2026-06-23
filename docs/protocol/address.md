# Protocol Specification: Addressing & Encoding

This document defines the cryptographic address formats, encoding, network prefixes, and checksum verification rules for the ARUNA Network.

## 1. Network Prefixes (Human Readable Part)
ARUNA uses distinct Human Readable Parts (HRP) in its addresses to enforce separation between network stages and prevent accidental cross-network transfers:

| Network Stage | Standard Wallet Prefix | Smart Contract Prefix |
| :--- | :--- | :--- |
| **Sumatera Testnet** | `sum1` | `sumc1` |
| **Kalimantan Testnet** | `kal1` | `kalc1` |
| **Sulawesi Testnet** | `sul1` | `sulc1` |
| **Papua Release Candidate**| `pap1` | `papc1` |
| **Jawa Mainnet** | `jaw1` | `jawc1` |

## 2. Derivation & Hashing Pipeline
Addresses represent public key hashes. The derivation path proceeds as follows:
1. **Key Generation:** A standard BIP32/44 wallet derives a private key and its corresponding public key (Ed25519 or secp256k1).
2. **Hash Computation:** Compute the BLAKE3 digest of the serialized public key and extract the first 20 bytes:
   `PublicKeyHash = BLAKE3(PublicKey)[0..20]` (resulting in a **20-byte digest**).
3. **Bech32m Encoding:** Encode the 20-byte `PublicKeyHash` using the Bech32m base32 character set, appending a checksum.

## 3. Bech32m Encoding Specification
* **Standard:** BIP-350 (Bech32m). The older BIP-173 (Bech32) standard is avoided due to the "length-extension" vulnerability in 0-value address checking.
* **Character Set:** Alphanumeric characters excluding `1`, `b`, `i`, and `o` (to avoid visual confusion with `l`, `1`, `0`, and `o`):
  `qpzry9x8gf2tvdw0s3jn54khce6mua7l`
* **Separator:** The constant character `1` acts as the boundary separator between the HRP and the payload data.
* **Checksum:** A 6-character BCH checksum is calculated by converting the HRP characters and data payload into base32 integers, computing a polymod check, and appending the checksum characters to the end.

## 4. Length and Formatting
* **Address Length:** ARUNA standard addresses are exactly **42 characters** long.
  * HRP (`jaw1`): 4 characters.
  * Separator (`1`): 1 character.
  * Encoded 20-byte hash (expanded to 32 base32 characters): 32 characters.
  * Checksum: 6 characters.
  * Total: $4 + 1 + 32 + 6 = 43$ characters (for contracts using `jawc1`, the length is 44 characters).
* **Validation Rules:**
  * All addresses are lowercase.
  * Address parsing libraries must reject any address containing mixed uppercase and lowercase characters.
