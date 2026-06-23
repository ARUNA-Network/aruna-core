# RFC-0012: Address Encoding Freeze

## Status
Accepted

## 1. Background
Public address formats, network stage prefixes (Human-Readable Parts or HRPs), and cryptographic validation rules represent the protocol's primary interface to wallets, explorers, exchange integrations, and stakers. Altering the address schema after the launch of the public testnet incurs extreme migration costs, risks transaction loss, and fragments client compatibility. This RFC officially freezes the address encoding specification.

## 2. Proposed Specification

### A. Immutable Network Separators and HRPs
The HRP prefixes for user wallets and contract identities are frozen as follows:

| Network Stage | Standard Wallet Prefix | Smart Contract Prefix |
| :--- | :--- | :--- |
| **Sumatera Testnet** | `sum` | `sumc` |
| **Kalimantan Testnet** | `kal` | `kalc` |
| **Sulawesi Testnet** | `sul` | `sulc` |
| **Papua Release Candidate**| `pap` | `papc` |
| **Jawa Mainnet** | `jaw` | `jawc` |

The separator character between the HRP and data parts is fixed as `1`.

### B. Payload Structure and Checksum
* **Payload Length:** The encoded data payload represents the 20-byte `PublicKeyHash` calculated via:
  $$\text{PublicKeyHash} = \text{RIPEMD160}(\text{SHA256}(\text{PublicKey}))$$
* **Padding Constraint:** The 20-byte payload is expanded to 5-bit base32 groups. For transaction payloads and EVM compatibility, this 20-byte hash is left-padded with zero bytes into a 32-byte field (bytes `0..12` are set to `0`, bytes `12..32` contain the hash).
* **Checksum Algorithm:** Enforced as BIP-350 Bech32m utilizing the polymod constant `0x2bc830f3`.

## 3. Migration & Security Implications
This freeze guarantees that any standard compliant library (e.g. Flutter wallets, web explorers) can rely on these exact HRPs and checksum behaviors from genesis onward. Address length remains constant at 42 characters for standard user wallets and 43 characters for contract wallets on the mainnet.
