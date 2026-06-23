# ADR-0006: Block Time 30 Seconds

## Status
Proposed

## Context
The block time of a blockchain defines how frequently transactions are ordered, processed, and added to the ledger. This parameter governs:
1. **Transaction Latency:** How fast a user's transaction is included in the blockchain state.
2. **Network Bandwidth:** How frequently peers must gossip block headers and transactions across the P2P layer.
3. **Uncle Block Rate:** The percentage of blocks mined concurrently that do not become part of the canonical chain (orphans/uncles).
4. **State Synchronization:** How easily low-power nodes (Raspberry Pi, Android) can sync the blockchain without falling behind.

ARUNA Network targets consumer internet connections (mobile networks, home DSL) and low-power ARM nodes.

## Problem
Selecting the wrong block time introduces severe operational challenges:
1. **Too Fast (e.g., <5 seconds):** Leads to a high rate of orphan blocks on mobile networks due to latency variance. Low-power nodes (Android smartphones) would spend too much CPU/battery processing block propagation, causing overheating and rapid battery drain.
2. **Too Slow (e.g., >10 minutes, like Bitcoin):** Results in a poor user experience. Transactions take too long to settle, making ARUNA unsuitable for payment applications or real-time DEX swaps.

We need a block time that balances transaction speed with network stability on consumer connections.

## Decision
We establish a target **Block Time of 30 Seconds**.

### Parameters & Implementations:
1. **Soft Finality (Confirmations):** 4 blocks (~2 minutes).
2. **Difficulty Adjustment Algorithm:** Weighted Moving Average (WMA) adjusting difficulty target *every block* to maintain the 30-second target block time.
3. **Block Header Fields:** The header contains a timestamp and a difficulty target used to calculate difficulty adjustments dynamically based on hash rate and block generation speeds.

## Alternatives
* **Alternative A: Fast Block Time (e.g., 5 seconds):** Rejected because mobile networks and home labs have high latency. A 5-second block time would create a massive number of uncle blocks, favoring high-bandwidth datacenters and excluding community nodes.
* **Alternative B: Slow Block Time (e.g., 60-120 seconds):** Rejected because transaction finality would take 5 to 10 minutes, which is too slow for consumer transactions and DEX swaps.

## Consequences
* **Positive:**
  * **Resiliency on Low-Bandwidth Networks:** 30 seconds provides enough time for block propagation to complete across home labs and mobile connections.
  * **Low Orphan Rate:** Significantly reduces the occurrence of conflicting blocks.
  * **Predictable Confirmation Time:** Users receive confirmation in under 2 minutes (4 blocks).
* **Negative:**
  * Interactive dApps (DEX) will have a 30-second latency before state changes are committed to a block, which is slower than chains like Solana or Arbitrum. However, this is a necessary trade-off to ensure decentralization and accessibility.

## Migration
Not applicable. The 30-second target block time is active from the genesis block.

## Security Impact
A stable block time prevents difficulty adjustment oscillation and difficulty shocks. The WMA algorithm prevents hash rate spikes from locking out low-power CPU miners by adapting difficulty targets immediately on each block transition. This maintains a predictable and secure block generation rate.
