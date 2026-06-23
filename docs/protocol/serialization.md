# Protocol Specification: Binary Serialization Standard

This document defines the binary encoding, serialization rules, and byte layouts for all consensus and networking messages in the ARUNA Network.

## 1. Serialization Standard: Bincode
To ensure execution determinism, low parsing latency, and zero payload size overhead, ARUNA adopts **Bincode** (version 1.3 or higher) as its primary binary serialization standard.
* **Justification:** Alternative standards like JSON are text-based (high parsing overhead, floating-point non-determinism). Protocol Buffers/gRPC or SSZ introduce complex compiler dependencies and schema management. Bincode maps directly to Rust structs and enforces tight, byte-exact layouts.

## 2. Integer and Data Endianness
To prevent cross-architecture divergence (e.g. between little-endian ARM CPUs and big-endian network transports):
* **Fixed Width:** All integers are serialized as fixed-width types: `u8`, `u16`, `u32`, `u64`, or `u128`.
* **Endianness:** All integers must be serialized in **Big-Endian (Network Byte Order)** format.
* **Floating-Points:** Floating-point numbers (`f32`, `f64`) are **strictly forbidden** in consensus-serialized structures. All decimal numbers must be represented as fixed-point integers (e.g. currency in micro-ARU `u64`).

## 3. Variable-Length Fields & Vectors
* **Length Prefixes:** Vectors, byte slices, and strings are serialized by prepending a **64-bit unsigned big-endian integer (`u64`)** representing the length of the collection, followed immediately by the raw elements.
* **Example Vector Serialization:**
  ```
  [ Length (u64 - 8 bytes) ] || [ Element 1 ] || [ Element 2 ] || ...
  ```

## 4. Enum Representation
Rust enums are serialized by prepending a **32-bit big-endian integer (`u32`)** representing the variant index (discriminant), starting at `0` for the first declared variant, followed immediately by the fields of that variant.

---

## 5. Binary Layout: Transaction Envelope
A complete transaction envelope is represented binary as:

| Field | Size | Data Type | Description |
| :--- | :--- | :--- | :--- |
| **Payload Length** | 8 bytes | `u64` | Big-endian length of the transaction payload |
| **Payload Data** | Variable | `[u8]` | The serialized transaction payload bytes |
| **Signature Type** | 1 byte | `u8` | `0` for Ed25519, `1` for secp256k1 |
| **Signature Bytes**| 64 or 65 bytes | `[u8]` | Cryptographic signature bytes |
