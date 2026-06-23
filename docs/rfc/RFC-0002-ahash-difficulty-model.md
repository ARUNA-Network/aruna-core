# RFC-0002: AHash Difficulty Adjustment Model

## Status
Accepted

## 1. Background
To maintain a stable block time of 30 seconds under fluctuating miner participation, the mining difficulty target must adapt dynamically. This RFC defines the difficulty adjustment algorithm parameters for the AHash proof-of-work mechanism.

## 2. Proposed Specification
We propose utilizing a **Weighted Moving Average (WMA)** algorithm adjusting difficulty on every block:

* **Window Size ($W$):** Last 120 blocks (~60 minutes of history).
* **Target Block Time ($T$):** 30 seconds.
* **Expected Window Time ($E$):**
  $$E = W \times T = 120 \times 30 = 3,600 \text{ seconds}$$
* **Actual Window Time ($A$):** The difference between the timestamp of block $N$ and block $N-120$.
* **Target Adjustment Formula:**
  $$\text{Target}_{N+1} = \text{Target}_N \times \frac{A}{E}$$

### Damping & Safety Guardrails:
To protect the chain against rapid hashrate spikes and difficulty oscillation:
* **Target Multiplier Bounds:** The adjustment factor $\frac{A}{E}$ is bounded between a minimum of **0.80** (difficulty increases by 25%) and a maximum of **1.25** (difficulty decreases by 20%) per block.
* **Time Spam Safeguard:** If $A$ is less than 900 seconds (15 minutes), $A$ is bound to 900 seconds to prevent malicious timestamps from artificially inflating difficulty targets.
