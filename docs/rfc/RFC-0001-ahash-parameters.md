# RFC-0001: AHash v1 Mining Algorithm Parameters

## Status
Accepted

## 1. Background
According to [ADR-0009](file:///home/coleallstar/Public/crypto-project/docs/adr/adr-0009-ahash-mining-algorithm.md), ARUNA Network utilizes a custom CPU-focused hashing pipeline named **AHash Specification v1** to support energy-efficient mining on commodity hardware (Android phones, Raspberry Pi, Mini PCs, and low-cost x86 CPUs).

The goal of this Request for Comments is to define the exact cryptographic parameters and thread constraints for the AHash execution pipeline, specifically focusing on the Argon2id memory-hard hashing stage.

## 2. Proposed Parameters

### A. Argon2id Configuration
Argon2id is a hybrid variant of Argon2 that combines the data-independent memory access of Argon2i (resisting side-channel cache attacks) with the data-dependent memory access of Argon2d (resisting time-memory trade-off GPU parallelization attacks).

We propose the following parameters for the Argon2id stage:
1. **Memory Cost (`m_cost`):** **16,384 KiB (16 MiB)**.
   * *Rationale:* 16 MiB is memory-hard enough to force GPU and ASIC implementations to allocate expensive physical memory blocks (preventing low-cost parallelization), yet small enough to execute in under 15 milliseconds on a single core of an ARMv8 Cortex-A53 processor (Cortex-A53 is common in Raspberry Pi 3/4 and low-end smartphones).
2. **Time Cost (`t_cost`):** **1 (single pass)**.
   * *Rationale:* Limits execution time and battery drain on mobile phones, keeping verification latency low for node verification loops.
3. **Parallelism (`p_cost`):** **1 (single lane/thread)**.
   * *Rationale:* Multi-thread Argon2 execution requires locking and coordination. By enforcing a single lane, we align with background screen-off mining constraints where a mobile miner runs on a single background core, leaving other device cores free for standard smartphone usage.

### B. AES Mixing Block Size
* **Block Size:** **1,024 bytes**.
* **Formula:** The intermediate digest from BLAKE3 is expanded into a 1,024-byte buffer. This buffer undergoes 12 rounds of AES block mixing utilizing native `AES-NI` (x86) and `AESE/AESMC` (ARMv8 Cryptography extensions) hardware instruction sets.

### C. ARM NEON Vectorization
* The final mixing stage compiles natively to **ARM NEON SIMD** vector registers (using `float32x4_t` or integer equivalents). If compiled on x86_64, the compiler must target standard `SSE2`/`AVX` registers, ensuring cross-platform determinism of vector rounding operations.

## 3. Discussion Points & Open Questions
1. **Memory Cost Scaling:** Should `m_cost` scale dynamically based on epoch height to track hardware memory cost drops?
   * *Draft Response:* No, dynamic memory cost scaling makes hardware target validations unstable and complicates light node checkpoint verifications. We recommend keeping `m_cost` fixed for Era 1.
2. **GPU Benchmark Checks:** Do we need to run GPU benchmarks to confirm that a 16 MiB memory footprint provides sufficient ASIC/GPU resistance?
   * *Draft Response:* Initial tests indicate that 16 MiB penalizes GPU performance due to memory bandwidth limits, but further benchmarking is scheduled during the Kalimantan Testnet (Year 2).
