# Economic Simulation: Miner Economics & Hashrate Spikes

This document models the mining reward dynamics and simulated chain behavior under sudden hashrate increases.

## 1. Hashing Rewards Base
* **PoW Reward Per Block:** 17.5 ARU.
* **Daily Minted PoW Rewards:**
  $$\text{DailyMinerRewards} = 2,880 \text{ blocks} \times 17.5 \text{ ARU} = 50,400 \text{ ARU}$$
* **Distribution Model:** Distributed to the miner solving the AHash puzzle for the block.

---

## 2. Simulation: Hashing Hashrate Spikes 100x
Assume a sudden, massive influx of hashing power (e.g., standard desktop groups or rented hashrate pools joining) resulting in a **100x increase in network hashrate**.

### A. Short-Term Protocol Impact (First 10 Seconds)
1. **Block Speed Acceleration:** The average time to find a block will temporarily collapse from 30 seconds to approximately **0.3 seconds**.
2. **Difficulty Adjustment Lag:** The Weighted Moving Average (WMA) difficulty adjustment algorithm updates target difficulty on every block.
   * To prevent difficulty shocks, target target adjustments are capped at a maximum of $\pm 25\%$ per block.
   * To adjust for a 100x hashrate increase, the difficulty must rise by a factor of 100.
   * Let $N$ be the number of blocks required to adjust difficulty:
     $$1.25^N = 100 \implies N = \frac{\log(100)}{\log(1.25)} \approx 20.6 \text{ blocks}$$
3. **Adjustment Duration:** It takes **21 blocks** for the difficulty to fully adapt. At 0.3-second block times during the transition, these 21 blocks are mined in **approximately 6.3 seconds**.
4. **Restoration:** After 6.3 seconds, difficulty target reaches equilibrium, and block times converge back to the stable **30-second target**.

---

## 3. Long-Term Economic Impacts

### A. Dilution of Consumer Hashrate Share
* The total daily minted miner rewards remain fixed at 50,400 ARU.
* A 100x hashrate increase means that the average yield per megahash drops by **100x**.
* **Impact on Mobile Miners:** Low-power devices (Android phones, Raspberry Pi) solo mining on the network will experience a 100x drop in their probability of solving a block, increasing reward variance to unacceptable levels.

### B. Mandatory Mitigations
To protect community miners under high hashrate competition:
1. **Transition to Pool Mining:** Mobile/low-power miners must run the **Miner-Light** client connected to decentralized community mining pools (using PPLNS payout schemes) to smooth reward distribution variance.
2. **Hybrid consensus protection:** If the hashrate spike represents a hostile 51% attack, the attacker still cannot reorganize or double-spend the chain because they lack validator signatures (>66.7% PoS stake) required to commit blocks, rendering a PoW-only attack economically futile.
