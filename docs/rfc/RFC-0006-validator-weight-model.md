# RFC-0006: Validator Weight Concentration Limits

## Status
Accepted

## 1. Background
To prevent plutocratic validator coalitions and ensure validation decentralization, this RFC proposes a mechanism to cap individual validator voting and reward weight.

## 2. Proposed Specification

### A. Validator Staked Weight Limit
* **Weight Limit Cap:** **10%** of the total network staked weight (including self-stake + delegated stake).
* **Formula:**
  $$\text{ValidatorWeight}_i = \min(\text{StakedVal}_i, \text{TotalStakedWeight} \times 10\%)$$

### B. Impact on Rewards & Voting
1. **Reward Payouts:** Any delegated stake that pushes a validator's total above the 10% cap is ignored for reward calculations. The validator and its delegators receive block reward distributions as if the validator's stake was exactly 10% of the network total.
2. **Voting Power:** The validator's voting power during checkpoint consensus and governance is capped at 10%.
3. **Ecosystem Incentive:** This rule incentivizes delegators to monitor validator weights and redirect their stake to smaller, high-performance validators to maximize their yields, fostering decentralization.
