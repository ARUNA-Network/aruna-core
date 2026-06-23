# ADR-0008: Bitcoin-Style Halving

## Status
Proposed

## Context
In a capped-supply blockchain protocol, new coins are introduced into circulation as block rewards to incentivize miners and stakers to secure the network. The rate at which these coins are emitted must be controlled to balance early bootstrapping incentives with long-term scarcity.

A proven model is the disinflationary halving schedule, where block rewards are cut in half at regular intervals, mimicking resource extraction constraints and preventing token hyperinflation.

## Problem
Designing an inappropriate emission model introduces several economic and security risks:
1. **Constant High Emission (Inflationary):** Floods the market with supply, depressing the value of the token and discouraging holders.
2. **Abrupt Reward Drop (Security Budget Drop):** If rewards drop too fast or unpredictably, miners may exit the network simultaneously, causing a hashrate crash that exposes the chain to 51% attacks.
3. **Complex Adjustments:** Dynamic emission formulas based on network conditions are difficult to test and can introduce consensus bugs or non-determinism across platforms.

We need a simple, mathematically deterministic emission schedule that establishes long-term scarcity while providing a predictable reward structure.

## Decision
We implement a **Bitcoin-Style Halving Model** for ARUNA.

### Emission Parameters:
* **Genesis Block Reward:** 25 ARU.
* **Block Time:** 30 Seconds.
* **Blocks Per Year:** 1,051,200 blocks.
* **Halving Interval:** 4 Years (exactly **4,204,800 blocks per era**).

### Reward Eras Breakdown:
* **Era 1 (Years 0–4):** 25.0000 ARU per block.
* **Era 2 (Years 4–8):** 12.5000 ARU per block.
* **Era 3 (Years 8–12):** 6.2500 ARU per block.
* **Era 4 (Years 12–16):** 3.1250 ARU per block.
* **Era 5 (Years 16–20):** 1.5625 ARU per block.
* *Emission continues halving every 4,204,800 blocks until the total supply approaches the 1,000,000,000 ARU cap.*

### Execution Rule:
Block rewards are calculated programmatically inside the consensus module based strictly on block height:
`Era = Height / 4,204,800`
`BlockReward = 25 >> Era` (using fixed-point arithmetic for precision).

## Alternatives
* **Alternative A: Linear Decay Model:** Block rewards decrease by a small fraction on every block instead of an abrupt halving. We rejected this because a linear decay is harder to represent deterministically across floating-point architectures and lacks the psychological coordination value of a distinct halving event.
* **Alternative B: Tail Emission (Infinity Inflation):** Block rewards never go to zero but remain at a fixed flat rate (e.g., Monero). We rejected this because it violates the immutable 1,000,000,000 ARU maximum supply rule.

## Consequences
* **Positive:**
  * **Hard Scarcity:** Guarantees disinflationary token economics.
  * **Deterministic Emission:** The total supply cap is mathematically verifiable by checking block height calculations.
  * **Early Bootstrapping:** Higher rewards in Era 1 incentivize early community miners and stakers.
* **Negative:**
  * **Fee Dependency:** After Era 5, the network must generate sufficient transaction volume to support miners and stakers through transaction fees alone as block rewards become negligible.

## Migration
Not applicable. The halving schedule is hardcoded into the genesis block execution logic.

## Security Impact
The programmatic halving calculation depends only on block height, ensuring deterministic calculation across ARM and x86 systems. There are no state lookups, external inputs, or oracle dependencies, making the emission schedule resistant to network manipulation or consensus divergence.
