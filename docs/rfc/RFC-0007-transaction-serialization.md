# RFC-0007: Transaction Serialization & Layouts

## Status
Accepted

## 1. Background
To ensure transaction propagation determinism, this RFC specifies the Bincode serialization configuration and binary field layouts for transaction envelopes.

## 2. Proposed Specification

### A. Bincode Configuration
All nodes must use Bincode configured as follows:
* **Endianness:** Big Endian (`bincode::options().with_big_endian()`).
* **Integer Encoding:** Fixed width (`bincode::options().with_fixint_encoding()`).
* **Limit:** Capped at 2 MB (`bincode::options().with_limit(2 * 1024 * 1024)`).

### B. Transaction Payload Binary Layout:
1. **Nonce:** 8 bytes (`u64` Big-Endian).
2. **Sender Address:** 32 bytes (`[u8; 32]`).
3. **Recipient Address:** 32 bytes (`[u8; 32]`).
4. **Amount:** 8 bytes (`u64` Big-Endian).
5. **Fee:** 8 bytes (`u64` Big-Endian).
6. **Gas Limit:** 8 bytes (`u64` Big-Endian).
7. **Gas Price:** 8 bytes (`u64` Big-Endian).
8. **Data Length:** 8 bytes (`u64` Big-Endian length prefix).
9. **Data:** Variable bytes.
