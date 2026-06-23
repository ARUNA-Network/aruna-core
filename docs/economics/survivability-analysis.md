# Economic Simulation: Price Independent Survivability Analysis

This document analyzes the survivability of miners, validators, and the network treasury under depressed exchange rate scenarios to ensure long-term protocol viability.

## 1. Core Simulation Assumptions
* **Daily Minting Rate:** 72,000 ARU (2,880 blocks/day $\times$ 25 ARU/block).
  * Miner Reward (70%): 50,400 ARU/day (17.5 ARU/block).
  * Validator Reward (25%): 18,000 ARU/day (6.25 ARU/block).
  * Treasury Reward (5%): 3,600 ARU/day (1.25 ARU/block).
* **Validator Count:** 100 active nodes.
* **Average Validator Hosting Cost:** $15.00 / month ($0.50 / day) for a standard 4-Core, 8GB RAM VPS.
* **Standard CPU Miner Power Footprint:** 10 Watts (Mini PC or ARM Server core).
* **Average Electricity Cost:** $0.12 per kWh ($0.0288 per day per active mining core).

---

## 2. Price Scenario Matrix

| Scenario | ARU Price | Daily Miner Yield (per core at 0.1% hashrate) | Daily Validator Yield (per node at 1% stake) | Daily Treasury Budget |
| :--- | :--- | :--- | :--- | :--- |
| **Scenario A** | $0.0001 | 50.4 ARU ($0.005) | 180 ARU ($0.018) | 3,600 ARU ($0.36) |
| **Scenario B** | $0.001 | 50.4 ARU ($0.050) | 180 ARU ($0.180) | 3,600 ARU ($3.60) |
| **Scenario C** | $0.01 | 50.4 ARU ($0.504) | 180 ARU ($1.800) | 3,600 ARU ($36.00) |
| **Scenario D** | $0.10 | 50.4 ARU ($5.040) | 180 ARU ($18.00) | 3,600 ARU ($360.00) |

---

## 3. Survivability Impact Analysis

### Scenario A: ARU = $0.0001 (Ultra-Low Price)
* **Miner Viability:** **PARTIAL / VOLUNTEER.**
  * *Analysis:* A single dedicated mining core consumes $0.0288/day in electricity but yields only $0.005 in value. Mining is unprofitable for commercial setups. However, because AHash is designed for ARM/Android background screen-off mining, users utilizing excess/zero-cost solar energy or running on idle home devices (Raspberry Pi/smartphones) will survive as hobbyist operators.
* **Validator Viability:** **VULNERABLE.**
  * *Analysis:* At $0.018/day in rewards, the monthly income ($0.54) fails to cover the $15.00 VPS cost. Professional node operators will shut down. The network must rely on founder-funded bootstrap nodes or stakers running nodes on home servers (Raspberry Pi/Mini PC) behind home connections to maintain liveness.
* **Treasury Viability:** **CRITICAL.**
  * *Analysis:* The daily treasury budget of $0.36 is insufficient to pay for active developers or infrastructure (RPC, DNS, Explorer hosting).
  * *Mitigation:* The protocol must freeze non-essential upgrades and run on volunteer infrastructure.

### Scenario B: ARU = $0.001 (Low Price Recovery)
* **Miner Viability:** **REGIONAL SURVIVABILITY.**
  * *Analysis:* Unprofitable in high-cost energy regions, but viable in areas with subsidized or low-cost energy (<$0.04/kWh) or on low-power mobile devices.
* **Validator Viability:** **SUBSIDIZED.**
  * *Analysis:* A monthly yield of $5.40 offset against $15.00 hosting results in a net loss of $9.60/month. Enthusiast validator stakers continue running nodes, but validator count may consolidate.
* **Treasury Viability:** **LIMITED.**
  * *Analysis:* The monthly budget of $108.00 covers basic domain names, backup RPC, and minimal server hosting, but cannot support core development.

### Scenario C: ARU = $0.01 (Mid-Tier Testnet Baseline)
* **Miner Viability:** **PROFITABLE.**
  * *Analysis:* Yields $0.50/day against $0.0288/day power cost. Miner participation rises, driving difficulty adjustment upward.
* **Validator Viability:** **PROFITABLE.**
  * *Analysis:* A validator node earns $54.00/month, yielding a net profit of $39.00/month after VPS hosting costs. Node count easily stays at the 100 validator cap.
* **Treasury Viability:** **SUSTAINABLE.**
  * *Analysis:* Monthly budget of $1,080.00. This is sufficient to pay for active explorer hosting, public node clusters, and core software updates.

### Scenario D: ARU = $0.10 (Target Baseline)
* **Miner Viability:** **HIGHLY PROFITABLE.**
  * *Analysis:* Commercial and hobbyist mining thrives globally. High incentive leads to mobile miner growth.
* **Validator Viability:** **HIGHLY PROFITABLE.**
  * *Analysis:* Net profit of $525.00/month per validator. High competition for active validator slots.
* **Treasury Viability:** **ROBUST.**
  * *Analysis:* Monthly budget of $1,0800.00, enabling funding of ecosystem grants, audits, and developer team compensation.

---

## 4. Adaptive System Adjustments (Under Scenario A & B)
If the token price stays depressed for more than 30 days, the protocol adapts through the following mechanisms:
1. **Dynamic MinFee Adjustment:** Under low token prices, the transaction BaseFeePerByte will dynamically increase (via governance consensus vote) to increase the cash yield from transaction fees, compensating for low inflation value.
2. **Validator Off-Chain Clustering:** Encourage stakers to move from high-cost AWS/Google Cloud instances to low-cost residential home nodes (e.g. Raspberry Pi 4/5) to lower validator operational expenses to near-zero.
3. **Treasury Reserve Allocation:** Utilize pre-mined bootstrap funds to subsidize essential RPC infrastructure.
