# ADR-0003: Rust As Primary Language

## Status
Proposed

## Context
A public blockchain protocol requires high performance, strict execution determinism, low resource footprints (especially for consumer devices like Android and Raspberry Pi), and high cryptographic safety. The language chosen to implement the core protocol must support compile-time safety checks to minimize runtime failures.

Additionally, the project is run under solo-founder constraints with AI assistance, meaning code readability, strict compiler diagnostics, and ecosystem package manager standardizations are crucial.

## Problem
Selecting a language that lacks memory safety, has a large garbage collection runtime, or produces non-deterministic cross-platform builds introduces key vulnerabilities:
1. **Memory Corruption:** Languages like C or C++ are prone to buffer overflows, double frees, and data races, which can crash nodes or lead to remote code execution (RCE) exploits.
2. **Resource Exhaustion:** High-level garbage-collected languages (like Go, Java, or C#) introduce unpredictable latency spikes (GC pauses) and consume significant memory, making background screen-off mining on Android smartphones or low-memory Raspberry Pi nodes unstable.
3. **Weak Compiler Support for Agents:** Dynamically typed languages (like Python or JavaScript) make it easier for AI agents to introduce subtle typing or architectural bugs that are only discovered at runtime.

We need a language that compiles to native code, offers mathematical compile-time guarantees, and operates efficiently on low-cost ARM and x86 hardware.

## Decision
We select **Rust (Stable)** as the primary language for all core protocol modules (`crates/*` and node CLI binaries).

### Reasons for Selecting Rust:
1. **Memory Safety Without Garbage Collection:** Rust's borrow checker enforces ownership and lifetime rules at compile time, eliminating null pointers, dangling references, and data races without requiring a memory-heavy garbage collector.
2. **Zero-Cost Abstractions:** Offers performance equivalent to C/C++, allowing optimized execution of cryptographic operations and the AHash mining algorithm.
3. **Cross-Compilation:** Rust's standard tooling makes cross-compiling to ARM64 (Linux servers, Android devices, Raspberry Pi) and x86_64 simple and reliable.
4. **Strong Compiler Diagnostics:** The compiler acts as a static analyzer, providing descriptive error messages. This allows AI agents to write safe code and quickly self-correct build failures.
5. **Ecosystem & Libraries:** The Cargo package manager simplifies workspace modularity. Key crates like `tokio` (async runtime), `serde` (serialization), and `libp2p` (peer-to-peer) are mature and performant.

### Coding Rules:
* **Unsafe Rust Constraint:** Avoid using `unsafe` blocks. If `unsafe` is necessary (e.g., for hardware-accelerated NEON/AES instructions), it must be documented with the exact safety justification, risks, and alternatives considered.
* **Stable Toolchain:** Use the stable Rust toolchain. Avoid nightly features unless required for hardware-specific optimizations.

## Alternatives
* **Alternative A: Go (Golang):** Used in Go-Ethereum (geth) and Cosmos. Go is simple and has a great networking stack. However, the runtime garbage collector introduces memory overhead and latency spikes that violate our Android screen-off mining constraints.
* **Alternative B: C++:** C++ offers absolute control and speed. However, it lacks memory safety guarantees, making it highly susceptible to exploits, and its build configuration (CMake/makefiles) is complex for automated AI agents.

## Consequences
* **Positive:**
  * Highly secure protocol foundation with compile-time memory safety.
  * Deterministic execution and tiny memory footprints, suitable for Tier 1 community hardware.
  * AI agents can easily compile and test code, correcting errors automatically using compiler logs.
* **Negative:**
  * Strict compiler checks result in a steeper learning curve for new community contributors.
  * Longer compilation times compared to Go or C, which can slow down CI/CD pipelines.

## Migration
Not applicable. The genesis protocol code will be built entirely in Rust.

## Security Impact
By utilizing Rust, we mathematically eliminate more than 70% of common security vulnerabilities (e.g., use-after-free, double free, buffer overflow, and data races). This is essential for a blockchain protocol operating in a zero-trust network.
