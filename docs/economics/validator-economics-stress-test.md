# Economic Simulation: Validator Staking Yield Stress-Testing

This document stress-tests the staking APR under different network participation rates and analyzes the corresponding economic impacts and risk mitigations.

## 1. Stress-Test Parameters
* **Fixed PoS Block Reward:** 6.25 ARU.
* **Annual PoS Inflation Payout:** 6,570,000 ARU (constant, does not scale with staked amount).
* **Validator Minimum Stake:** 10,000 ARU.

---

## 2. Yield Stress-Test Matrix
We calculate the resulting annual staking yields across different total staked weight thresholds:

| Case | Total Network Stake (ARU) | Staking APR (%) | Monthly Yield per Validator (at 10k stake) |
| :--- | :--- | :--- | :--- |
| **Case A** | $1,000,000$ (1M) | $657.00\%$ | $5,475 \text{ ARU}$ |
| **Case B** | $10,000,000$ (10M) | $65.70\%$ | $547.5 \text{ ARU}$ |
| **Case C** | $50,000,000$ (50M) | $13.14\%$ | $109.5 \text{ ARU}$ |
| **Case D** | $100,000,000$ (100M) | $6.57\%$ | $54.75 \text{ ARU}$ |
| **Case E** | $200,000,000$ (200M) | $3.28\%$ | $27.37 \text{ ARU}$ |

---

## 3. Inflation & Decentralization Analysis

### A. The "Massive Inflation Pressure" Myth
While a **657% APR** seems extremely high, it does **not** create hyperinflationary pressure on the overall coin supply. Because the block reward is fixed at 6.25 ARU per block, the total number of new coins minted for staking is strictly capped at **6,570,000 ARU per year**, regardless of whether the total stake is 1M ARU or 200M ARU.
* **Result:** The overall supply inflation is strictly bounded. The high APR simply represents a concentrated yield awarded to early infrastructure adopters when validation participation is low.

### B. The Real Risk: Supply Centralization
The critical threat of Case A (657% APR) is that the 100 early validators will collectively absorb a large share of the circulating supply. Within one year, they will earn 6.57M ARU, representing **~14.5% of the total circulating supply** (45M ARU).
* **Risk:** This can lead to validator oligopoly, where early nodes dominate future PoS consensus and governance proposal weights.

### C. Codified Mitigations
To prevent early validator centralization:
1. **Validator Staking Cap (10%):** No single validator node may control >10% of total staked weight. Any delegation exceeding 10% is stripped of reward weight, forcing delegators to seek smaller nodes.
2. **Offline Loss of Rewards:** Offline validators lose 100% of rewards immediately, encouraging stakers to delegate to reliable, decentralized operators.
3. **Founder Lockups:** Zero founder coins participate in validation or governance during the 48-month vesting period, preventing team centralizations.
