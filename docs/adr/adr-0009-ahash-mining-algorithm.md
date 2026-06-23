# ADR-0009: AHash Mining Algorithm

## Status
Proposed

## Context
The Proof of Work component of ARUNA's hybrid consensus requires a hashing algorithm to secure block headers. Unlike mainstream blockchains that utilize ASIC-friendly hash algorithms (e.g., SHA-256 for Bitcoin or Ethash for Ethereum Classic), ARUNA's mission is to empower ordinary users operating low-cost commodity devices (Android smartphones, Raspberry Pi, Mini PCs, and x86 CPUs).

The mining algorithm must be designed to run efficiently on CPU architectures (particularly ARM64 and ARM NEON instructions) while actively resisting ASIC parallelization and GPU dominance.

## Problem
Standard mining algorithms fail to meet ARUNA's accessibility requirements:
1. **ASIC Centralization:** Simple, memory-light hashing algorithms (like SHA-256 or Scrypt) are easily implemented on custom ASIC hardware, which is exponentially faster and more energy-efficient than consumer CPUs, centralizing mining power.
2. **GPU Domination:** Hashing algorithms that require simple mathematical operations with moderate memory footprints (like Ethash) are heavily dominated by GPU farms, pricing out CPU miners.
3. **Android Incompatibility:** Memory-hard or cache-heavy algorithms designed for high-end server CPUs (like RandomX) require massive L3 cache sizes (2MB+ per mining thread) and high RAM bandwidth, which are absent on standard mobile/ARM processors. This causes mobile devices to overheat and crash.

We need a hashing pipeline that is memory-hard but cache-friendly, utilizing common cryptographic primitives native to mobile and consumer hardware.

## Decision
We design and implement **AHash Specification v1** as the official Proof of Work mining algorithm.

### AHash Pipeline:
The input to AHash is the 80-byte block header. The pipeline consists of the following sequential stages:
```
Block Header (80 bytes)
      │
      ▼
   BLAKE3          (Fast, cryptographically secure initial digest)
      │
      ▼
 AES Mixing Stage  (Hardware-accelerated AES instruction set block mix)
      │
      ▼
Argon2 Memory      (Argon2id configuration optimized for mobile RAM size
 Expansion          and energy consumption)
      │
      ▼
 ARM NEON Layer    (Vectorized mixing stage utilizing SIMD/NEON on ARM64)
      │
      ▼
  Final Digest
```

### Core Pipeline Specifications:
1. **BLAKE3:** Generates a highly secure, fast initial 256-bit digest. It is selected over SHA-2 or SHA-3 due to its superior speed on both 32-bit and 64-bit platforms.
2. **AES Mixing Stage:** Utilizes hardware-accelerated AES instructions (AES-NI on x86, ARMv8 Cryptography extensions on ARM). Because AES hardware is built natively into almost all modern mobile and consumer CPUs, this provides a zero-cost speedup for target platforms while penalizing custom ASICs that do not include standardized AES blocks.
3. **Argon2id Memory Expansion:** We configure Argon2id (selected for its memory hardness and resistance to side-channel attacks) with parameters optimized for consumer hardware (e.g., `m_cost = 16384` [16 MB], `t_cost = 1`, `p_cost = 1`). This is memory-hard enough to deter GPU/ASIC pipelines without exceeding the RAM limits of low-cost Android and Raspberry Pi devices.
4. **ARM NEON Optimization Layer:** Incorporates vectorization utilizing ARM SIMD (NEON) instructions on ARM64, and SSE/AVX vectorizations on x86_64, maximizing CPU efficiency.

## Alternatives
* **Alternative A: RandomX (Monero):** Excellent CPU mining algorithm that executes random code. However, it requires a minimum of 2 GB of RAM per dataset and at least 2 MB of L3 cache per thread. This makes it impossible to mine on mobile devices and low-cost Raspberry Pi boards with limited cache sizes.
* **Alternative B: Sha-256:** Completely ASIC-friendly and consumes significant energy, violating the community ownership and accessibility philosophy.

## Consequences
* **Positive:**
  * **ASIC Resistance:** The combination of AES hardware acceleration, Argon2id memory hardness, and vector instructions makes it extremely expensive to build competitive ASICs for AHash.
  * **ARM & Android Friendly:** Runs efficiently on ARMv8 processors.
  * **Predictable Thermal Footprint:** The algorithm configuration is tuned to maintain safe operating temperatures on mobile devices under background thread limits.
* **Negative:**
  * Requires custom implementation and benchmarking across diverse CPU hardware to prevent cross-architecture verification bugs.

## Migration
Not applicable. AHash v1 is the consensus hashing algorithm at block height 0.

## Security Impact
AHash prevents hash rate centralizations by ensuring that consumer devices remain competitive. The hardware AES and vector instruction requirements ensure that any attacker attempting to mine on cloud server VMs faces performance penalties, favoring local physical hardware operators.
