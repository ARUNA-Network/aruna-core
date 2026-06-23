# ADR-0013: Address Format

## Status
Proposed

## Context
Public addresses are human-readable identifiers that represent accounts on the blockchain. They are used to receive transactions, deploy smart contracts, delegate staking, and cast votes.

The address format must satisfy these properties:
1. **Human-Friendliness:** Simple to write, copy, read, and verify, minimizing copy-paste errors.
2. **Error Detection:** Built-in error-correction coding to detect typing errors before transactions are sent.
3. **Network Distinction:** Clear prefixes to prevent users from accidentally sending mainnet funds to a testnet address (and vice versa).
4. **Tooling Compatibility:** Mobile-friendly, QR-code-friendly, and compatible with EVM address structures.

## Problem
Using standard hexadecimal addresses (like Ethereum's `0x...`) or raw base58 addresses (like Bitcoin's legacy formats) introduces several issues:
1. **Replay Across Networks:** An address format that does not specify the network prefix allows a user to paste a Sumatera Testnet address into a Jawa Mainnet wallet. If they sign a transaction, it can be replayed on mainnet, resulting in financial loss.
2. **Lack of Checksums:** Hexadecimal formats do not have built-in error correction (unless case-checksums are used, which are frequently ignored by developers). A single character typo will result in funds being lost forever.
3. **Visual Confusion:** It is difficult for non-technical users to distinguish standard wallets from smart contract wallets or system treasury accounts, leading to confusion.

We need a standardized, checksummed address format that explicitly states the network and account type.

## Decision
We adopt **Bech32m Encoding** (BIP-173 and BIP-350 specifications) as the official address format for ARUNA Network.

### Address Structure:
Every ARUNA address follows the pattern:
`[Human Readable Part (HRP)][Separator (1)][Data Part]`

The HRP specifies the network prefix and account type:
* **Sumatera Testnet Standard:** `sum1`
* **Kalimantan Testnet Standard:** `kal1`
* **Sulawesi Testnet Standard:** `sul1`
* **Papua Release Candidate Standard:** `pap1`
* **Jawa Mainnet Standard:** `jaw1`

### Smart Contract Address Format:
To distinguish smart contracts from standard user wallets, we insert `c1` as the separator:
* **Jawa Mainnet Contract Prefix:** `jawc1` (e.g., `jawc1[encoded public key hash]`).
* **Sumatera Testnet Contract Prefix:** `sumc1` (e.g., `sumc1[encoded public key hash]`).

### Address Derivation Formula:
1. Generate the public key (using Ed25519 or secp256k1).
2. Calculate the 160-bit hash of the public key by taking the first 20 bytes of its BLAKE3 hash:
   `PublicKeyHash = BLAKE3(PublicKey)[0..20]`
3. Encode the `PublicKeyHash` using the Bech32m base32 character set, appending the 6-character error-detection checksum.
4. Prepend the corresponding network prefix (e.g., `jaw1` or `jawc1`).

## Alternatives
* **Alternative A: Hexadecimal (0x-prefix):** Rejected because it lacks built-in error-correction checksums and does not provide distinct network separation prefixes, increasing the risk of user errors.
* **Alternative B: Base58Check (Bitcoin-style):** Standard format, but Base58 is less mobile-friendly (lacks uppercase characters, which makes QR code compression less efficient) and has weaker checksum guarantees compared to Bech32m.

## Consequences
* **Positive:**
  * **Accidental Cross-Network Protection:** Mainnet wallets will fail validation if a user attempts to send funds to a testnet address (e.g., trying to send to `sum1...` on a `jaw1...` node).
  * **Typo Protection:** Bech32m detects up to 4 character errors and guarantees detection of single-character typos, preventing loss of funds.
  * **QR Code Efficiency:** Bech32m uses lowercase alphanumeric characters, which compile into smaller, highly scannable QR codes on mobile screens.
* **Negative:**
  * Requires custom parsing and validation libraries in the SDKs (Rust, TypeScript, Dart) to convert Bech32m addresses back to raw public key hashes for EVM execution.

## Migration
Not applicable. The Bech32m address format is the genesis standard for all accounts.

## Security Impact
Enforcing Bech32m address validation at the transaction construction layer prevents users from sending assets to invalid addresses or mismatched networks. The distinct contract prefix (`jawc1`) alerts wallets to prompt users with contract interaction warnings, reducing phishing risks.
