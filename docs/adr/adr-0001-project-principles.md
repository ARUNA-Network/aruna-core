# ADR-0001: Project Principles

## Status
Proposed

## Context
ARUNA Network is conceived as the first globally adopted public blockchain originating from Indonesia. The blockchain space has historically been dominated by high-cost, high-overhead systems that favor large capital allocations, enterprise-grade validation infrastructure, and specialized hardware (ASICs and GPU farms). This has resulted in a gradual centralization of validator nodes and mining pools, locking out average participants who only own commodity hardware.

To solve this, ARUNA aims to build a public blockchain designed for fair and accessible participation through affordable and widely available consumer devices such as Android smartphones, Raspberry Pi computers, Mini PCs, and low-cost ARM/x86 servers. The core vision is encapsulated in the dual slogans:
* *"Dari Rakyat. Oleh Rakyat. Untuk Rakyat."*
* *"Mine Anywhere. Owned By Everyone."*

Because this is a long-term decentralized project, we must establish a core set of non-negotiable principles that protect the protocol from centralized takeover, founder capture, economic instability, and developer convenience.

## Problem
Without a set of binding, immutable architectural principles, the protocol faces significant risks over its lifecycle:
1. **Developer Convenience:** Engineers may make decisions that optimize for short-term shipping speeds but introduce long-term technical debt or centralized dependency.
2. **Economic Deviations:** Pressure from validators or speculative markets might lead to attempts to adjust block rewards, maximum token supply, or founder allocations.
3. **Consensus Compromise:** Decentralization might be sacrificed for throughput, leading to validation architectures that exclude ordinary consumer devices.
4. **Founder Dictatorship:** A single compromised account or private key could take over governance, protocol upgrades, or treasury funds.

We need a formal decision document to codify the absolute priorities and zero-tolerance boundaries of the project.

## Decision
We define the foundational principles of the ARUNA Network as follows:

### 1. Absolute Priority Order
In any design conflict, priorities are ranked strictly as:
1. **Security:** Zero trust architecture, localized private keys, and validation of all transactions.
2. **Consensus Correctness:** Deterministic transitions across x86, ARM, and Android platforms.
3. **Economic Sustainability:** Predictable token inflation and self-sustaining treasury.
4. **Decentralization:** Preventing single-entity or mining pool domination.
5. **Maintainability:** Simplicity over cleverness, supporting a solo founder.
6. **Reliability:** Tolerating network partitions and peer dropouts.
7. **Scalability:** Transaction capacity growth without raising hardware costs.
8. **Performance:** Efficient execution on commodity devices.
9. **Developer Experience:** Easy-to-read APIs and clean workspace.
10. **Feature Speed:** Features are shipped only when security and correctness are verified.

### 2. Zero-Tolerance Rules
The protocol will never allow:
* Breaking consensus safety.
* Hidden inflation or emergency minting.
* Founder privilege, administrative backdoors, or emergency bypasses.
* Undocumented protocol behavior.
* Protocol/consensus logic embedded in UI applications (all logic must live in core crates).
* Magic numbers or hardcoded secrets/peers/ports.
* Insecure or custom cryptographic primitives.

### 3. AI Agent Native Strategy
Because the developer is a solo founder, the engineering workflow must be optimized for multi-agent autonomous engineering. Every module must maintain clean documentation, clear API borders, and high unit test coverage (80% minimum overall, 95%+ for consensus) to facilitate AI code generation, verification, and auditing.

## Alternatives
* **Alternative A: Venture-Backed Speed-First Model:** Prioritize development speed, high performance, and marketing. We rejected this because VC funding introduces centralized pressure and forces speculative economics.
* **Alternative B: Pure Proof of Work (Bitcoin-style) or Pure Proof of Stake (Ethereum-style):** We rejected these because pure PoW favors ASIC datacenters, while pure PoS favors capital-rich stakers (plutocracy). A hybrid consensus is required to maintain ARM accessibility.

## Consequences
* **Positive:**
  * Ensures ARUNA remains community-owned and accessible to low-cost hardware.
  * Protects the protocol from structural capture.
  * Clarifies architectural constraints for future developers.
* **Negative:**
  * Development speed will be slower, as security, documentation, and audits are required before execution.
  * Performance optimizations must remain within the limits of low-cost hardware.

## Migration
Not applicable. These principles define the genesis state of the project.

## Security Impact
 Codifying these principles establishes a zero-trust development culture. Unsafe Rust is banned unless heavily justified, and the founder's key cannot override treasury or consensus parameters. This mitigates single-point-of-failure vulnerabilities at the human organization layer.
