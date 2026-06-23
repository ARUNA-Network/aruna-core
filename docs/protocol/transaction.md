# Protocol Validation: Transaction Specification

This document details the transaction structure, validation, fee models, and signing specifications for the ARUNA Network.

## 1. Transaction Fields
An ARUNA transaction is divided into the **Payload** (unsigned transaction details) and the **Envelope** (payload plus signatures and metadata).

### Transaction Payload Fields:
1. **Nonce (8 bytes / `u64`):** A counter representing the total number of transactions sent from the sender's account. This prevents double-spending and replay attacks.
2. **Sender Address (32 bytes / `[u8; 32]`):** The Bech32m-decoded address of the sender.
3. **Recipient Address (32 bytes / `[u8; 32]`):** The Bech32m-decoded address of the recipient.
4. **Amount (8 bytes / `u64`):** The value of ARU coins transferred (expressed in micro-ARU: $1 \text{ ARU} = 1,000,000 \text{ micro-ARU}$).
5. **Fee (8 bytes / `u64`):** The transaction fee offered to miners and validators (micro-ARU).
6. **Gas Limit (8 bytes / `u64`):** The maximum EVM gas allowed for contract deployment or execution.
7. **Gas Price (8 bytes / `u64`):** The fee offered per unit of gas.
8. **Data Part:** Optional variable-length bytes (used for smart contract bytecode or parameters).

## 2. Nonce Management
* **Strict Increment:** For a transaction to be valid, its nonce must equal `Sender_Account.nonce + 1`.
* **Sequential Ordering:** Transactions from the same account in the mempool are sorted and processed in ascending order of their nonces.
* **Double Spend Protection:** If a transaction with nonce `N` is committed, any transaction with nonce `N` or lower from that account is rejected.

## 3. Transaction Fee Model
All transactions require fees. The network enforces a **Minimum Fee Floor** to deter mempool spam:
* **Calculation Formula:**
  $$\text{MinFee} = \text{BaseFeePerByte} \times \text{TransactionSizeInBytes}$$
* **Default BaseFeePerByte:** 10 micro-ARU per byte.
* **Mempool Prioritization:** The mempool prioritizes transactions offering higher `GasPrice` or higher fee-per-byte density.
* **No Fee Burning:** 100% of transaction fees are distributed: 70% to the block miner, 25% to validators, and 5% to the Treasury.

## 4. Gas Model (EVM Transactions)
For smart contract operations, execution resource consumption is managed by EVM Gas:
* **Intrinsic Gas Cost:** Every EVM transaction has an base cost of **21,000 gas**. If the transaction contains data bytes, an additional **16 gas per non-zero byte** (4 gas per zero byte) is charged.
* **Block Gas Limit:** Capped at **30,000,000 gas** per block.
* **Execution Limit:** If a transaction runs out of gas during execution:
  1. All state changes are reverted.
  2. The fee is still consumed and awarded to miners/validators to prevent infinite loop DoS attacks.

## 5. Signature Schemes
ARUNA natively supports dual signature verification:
* **Standard Wallets (Ed25519):** Used for standard coin transfers, staking, and governance votes. Fast, secure, and mobile-friendly.
* **EVM Wallets (secp256k1):** Required for smart contract deployments, contract executions, and Web3 tool compatibility (MetaMask).

## 6. Serialization
* **Standard:** **Bincode** for binary network propagation.
* **Envelope Layout:**
  `TransactionEnvelope = (PayloadBytes || SignatureType [1 byte] || SignatureBytes [64/65 bytes])`
* **Parsing:** The node deserializes the envelope, reads the `SignatureType` byte to determine the cryptographic algorithm, and verifies the signature against the hash of the `PayloadBytes`.
