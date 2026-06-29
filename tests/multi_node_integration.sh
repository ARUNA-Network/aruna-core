#!/usr/bin/env bash
# System-level Integration Test for ARUNA Multi-Node Setup
# Spawns 3 node processes (Node A <-> Node B <-> Node C)
# Verifies:
# 1. P2P Handshake & Peer Discovery
# 2. Transaction Propagation
# 3. Block Propagation
# 4. Chain Synchronization (Catching up after being offline)

set -euo pipefail

echo "========================================================="
echo "🚀 STARTING ARUNA MULTI-NODE INTEGRATION TEST"
echo "========================================================="

# Helper to assert string contains a pattern
assert_contains() {
    local data="$1"
    local pattern="$2"
    local msg="$3"
    if [[ "$data" != *"$pattern"* ]]; then
        echo "❌ FAIL: $msg"
        echo "Expected pattern: $pattern"
        echo "Received data: $data"
        exit 1
    fi
}

# Helper to assert two strings are equal
assert_eq() {
    local val1="$1"
    local val2="$2"
    local msg="$3"
    if [[ "$val1" != "$val2" ]]; then
        echo "❌ FAIL: $msg"
        echo "Value 1: $val1"
        echo "Value 2: $val2"
        exit 1
    fi
}

# Cleanup function to kill background processes and clean directories
cleanup() {
    echo "🧹 Cleaning up background processes and temporary directories..."
    kill $NODE_A_PID $NODE_B_PID $NODE_C_PID 2>/dev/null || true
    wait $NODE_A_PID $NODE_B_PID $NODE_C_PID 2>/dev/null || true
    rm -rf ./data_sumatera_9010 ./data_sumatera_9011 ./data_sumatera_9012
    rm -f test_tx.json submit_res.json
    echo "✨ Cleanup complete."
}
trap cleanup EXIT

# ── PREPARATION ───────────────────────────────────────────────────────────────
echo "📦 Cleaning up existing data directories..."
rm -rf ./data_sumatera_9010 ./data_sumatera_9011 ./data_sumatera_9012
rm -f test_tx.json submit_res.json node_a_integ.log node_b_integ.log node_c_integ.log

echo "🔨 Building aruna-node binary and tools..."
CXXFLAGS="-include cstdint" cargo build --workspace

# ── STARTING NODE A ───────────────────────────────────────────────────────────
echo "🟢 Starting Node A (P2P Port: 9010, RPC Port: 8090, Block Time: 2s)..."
./target/debug/aruna-node --p2p-port 9010 --rpc-port 8090 --block-time 2 > node_a_integ.log 2>&1 &
NODE_A_PID=$!

echo "⏳ Waiting for Node A RPC server to start..."
for i in {1..15}; do
    if curl -s http://127.0.0.1:8090/status > /dev/null; then
        echo "Node A RPC is up!"
        break
    fi
    sleep 1
done

if ! curl -s http://127.0.0.1:8090/status > /dev/null; then
    echo "❌ ERROR: Node A failed to start. Logs:"
    cat node_a_integ.log
    exit 1
fi

# ── STARTING NODE B ───────────────────────────────────────────────────────────
echo "🟢 Starting Node B (P2P Port: 9011, RPC Port: 8091, Bootstrapping to Node A)..."
./target/debug/aruna-node --p2p-port 9011 --rpc-port 8091 --peer 127.0.0.1:9010 --block-time 3600 > node_b_integ.log 2>&1 &
NODE_B_PID=$!

echo "⏳ Waiting for Node B RPC server to start..."
for i in {1..15}; do
    if curl -s http://127.0.0.1:8091/status > /dev/null; then
        echo "Node B RPC is up!"
        break
    fi
    sleep 1
done

if ! curl -s http://127.0.0.1:8091/status > /dev/null; then
    echo "❌ ERROR: Node B failed to start. Logs:"
    cat node_b_integ.log
    exit 1
fi

# ── STARTING NODE C ───────────────────────────────────────────────────────────
echo "🟢 Starting Node C (P2P Port: 9012, RPC Port: 8092, Bootstrapping to Node B)..."
./target/debug/aruna-node --p2p-port 9012 --rpc-port 8092 --peer 127.0.0.1:9011 --block-time 3600 > node_c_integ.log 2>&1 &
NODE_C_PID=$!

echo "⏳ Waiting for Node C RPC server to start..."
for i in {1..15}; do
    if curl -s http://127.0.0.1:8092/status > /dev/null; then
        echo "Node C RPC is up!"
        break
    fi
    sleep 1
done

if ! curl -s http://127.0.0.1:8092/status > /dev/null; then
    echo "❌ ERROR: Node C failed to start. Logs:"
    cat node_c_integ.log
    exit 1
fi

# Give extra time for libp2p connection discovery and handshakes to finish
echo "⏳ Waiting 6 seconds for P2P peer discovery and handshake propagation..."
sleep 6

# ── STEP 1: VERIFY HANDSHAKE & PEER DISCOVERY ────────────────────────────────
echo "🔍 STEP 1: Verifying P2P Handshake & Peer Discovery..."

METRICS_A=$(curl -s http://127.0.0.1:8090/metrics)
METRICS_B=$(curl -s http://127.0.0.1:8091/metrics)
METRICS_C=$(curl -s http://127.0.0.1:8092/metrics)

PEER_COUNT_A=$(echo "$METRICS_A" | grep -oE '"peer_count":[0-9]+' | cut -d: -f2)
PEER_COUNT_B=$(echo "$METRICS_B" | grep -oE '"peer_count":[0-9]+' | cut -d: -f2)
PEER_COUNT_C=$(echo "$METRICS_C" | grep -oE '"peer_count":[0-9]+' | cut -d: -f2)

echo "Node A Peer Count: $PEER_COUNT_A"
echo "Node B Peer Count: $PEER_COUNT_B"
echo "Node C Peer Count: $PEER_COUNT_C"

if [ "$PEER_COUNT_A" -lt 1 ] || [ "$PEER_COUNT_B" -lt 1 ] || [ "$PEER_COUNT_C" -lt 1 ]; then
    echo "❌ FAIL: Peer discovery check failed. Nodes do not see each other."
    exit 1
fi
echo "✅ Handshake & Peer Discovery Successful!"

# ── STEP 2: VERIFY TRANSACTION PROPAGATION ────────────────────────────────────
echo "🔍 STEP 2: Verifying Transaction Propagation..."

echo "Preparing test transaction..."
CXXFLAGS="-include cstdint" cargo run -p aruna-testing --bin mempool_test_prep

# Submit the transaction to Node C
echo "Submitting transaction to Node C..."
curl -s -X POST -H "Content-Type: application/json" -d @test_tx.json http://127.0.0.1:8092/tx > submit_res.json
cat submit_res.json

assert_contains "$(cat submit_res.json)" "success" "Transaction submission response must be success"

# Extract tx hash
TX_HASH=$(grep -oE '[a-f0-9]{64}' submit_res.json | head -n1)
echo "Submitted Tx Hash: $TX_HASH"

echo "⏳ Waiting 4 seconds for P2P transaction gossip to propagate to Node A..."
sleep 4

# Query Node A for the transaction status (it should be in A's mempool as "pending")
echo "Querying Node A for transaction status..."
TX_A_RES=$(curl -s http://127.0.0.1:8090/transaction/$TX_HASH)
echo "Node A Response: $TX_A_RES"

assert_contains "$TX_A_RES" "transaction" "Node A must have received the transaction"
echo "✅ Transaction Gossip & Propagation Successful!"

# ── STEP 3: VERIFY BLOCK PROPAGATION ──────────────────────────────────────────
echo "🔍 STEP 3: Verifying Block Propagation..."

echo "⏳ Waiting 6 seconds for blocks to be produced and gossiped across all nodes..."
sleep 6

STATUS_A=$(curl -s http://127.0.0.1:8090/status)
STATUS_B=$(curl -s http://127.0.0.1:8091/status)
STATUS_C=$(curl -s http://127.0.0.1:8092/status)

HEIGHT_A=$(echo "$STATUS_A" | grep -oE '"height":[0-9]+' | cut -d: -f2)
HEIGHT_B=$(echo "$STATUS_B" | grep -oE '"height":[0-9]+' | cut -d: -f2)
HEIGHT_C=$(echo "$STATUS_C" | grep -oE '"height":[0-9]+' | cut -d: -f2)

echo "Heights -> Node A: $HEIGHT_A, Node B: $HEIGHT_B, Node C: $HEIGHT_C"

if [ "$HEIGHT_A" -lt 1 ]; then
    echo "❌ FAIL: No blocks were produced."
    exit 1
fi

assert_eq "$HEIGHT_A" "$HEIGHT_B" "Node A and Node B height must be equal"
assert_eq "$HEIGHT_A" "$HEIGHT_C" "Node A and Node C height must be equal"

TIP_A=$(curl -s http://127.0.0.1:8090/chain/tip)
TIP_B=$(curl -s http://127.0.0.1:8091/chain/tip)
TIP_C=$(curl -s http://127.0.0.1:8092/chain/tip)

HASH_A=$(echo "$TIP_A" | grep -oE '"hash":"[a-f0-9]+"' | cut -d: -f2 | tr -d '"')
HASH_B=$(echo "$TIP_B" | grep -oE '"hash":"[a-f0-9]+"' | cut -d: -f2 | tr -d '"')
HASH_C=$(echo "$TIP_C" | grep -oE '"hash":"[a-f0-9]+"' | cut -d: -f2 | tr -d '"')

echo "Tips -> Node A: $HASH_A, Node B: $HASH_B, Node C: $HASH_C"
assert_eq "$HASH_A" "$HASH_B" "Node A and Node B tip hash must match"
assert_eq "$HASH_A" "$HASH_C" "Node A and Node C tip hash must match"

echo "✅ Block Gossip & Propagation Successful!"

# ── STEP 4: VERIFY CHAIN SYNCHRONIZATION ──────────────────────────────────────
echo "🔍 STEP 4: Verifying Chain Synchronization..."

echo "🛑 Shutting down/killing Node C..."
kill $NODE_C_PID
wait $NODE_C_PID 2>/dev/null || true

echo "⏳ Waiting 6 seconds for Node A and Node B to produce new blocks..."
sleep 6

STATUS_A_NEW=$(curl -s http://127.0.0.1:8090/status)
HEIGHT_A_NEW=$(echo "$STATUS_A_NEW" | grep -oE '"height":[0-9]+' | cut -d: -f2)
echo "New Node A height: $HEIGHT_A_NEW"

if [ "$HEIGHT_A_NEW" -le "$HEIGHT_A" ]; then
    echo "❌ FAIL: Node A failed to produce new blocks after Node C was stopped."
    exit 1
fi

echo "🟢 Restarting Node C (fresh database clean check and synchronization test)..."
# Clean Node C database to make sure it performs a full synchronization from Node B
rm -rf ./data_sumatera_9012

./target/debug/aruna-node --p2p-port 9012 --rpc-port 8092 --peer 127.0.0.1:9011 --block-time 3600 > node_c_restarted.log 2>&1 &
NODE_C_PID=$!

echo "⏳ Waiting for restarted Node C RPC server to start..."
for i in {1..15}; do
    if curl -s http://127.0.0.1:8092/status > /dev/null; then
        echo "Node C RPC is back up!"
        break
    fi
    sleep 1
done

echo "⏳ Waiting 8 seconds for Node C to sync to Node B/A's height..."
sleep 8

STATUS_A_FINAL=$(curl -s http://127.0.0.1:8090/status)
HEIGHT_A_FINAL=$(echo "$STATUS_A_FINAL" | grep -oE '"height":[0-9]+' | cut -d: -f2)
STATUS_C_FINAL=$(curl -s http://127.0.0.1:8092/status)
HEIGHT_C_FINAL=$(echo "$STATUS_C_FINAL" | grep -oE '"height":[0-9]+' | cut -d: -f2)

echo "Final Heights -> Node A: $HEIGHT_A_FINAL, Node C: $HEIGHT_C_FINAL"
assert_eq "$HEIGHT_C_FINAL" "$HEIGHT_A_FINAL" "Node C must catch up to Node A's current height"

TIP_A_FINAL=$(curl -s http://127.0.0.1:8090/chain/tip)
TIP_C_FINAL=$(curl -s http://127.0.0.1:8092/chain/tip)
HASH_A_FINAL=$(echo "$TIP_A_FINAL" | grep -oE '"hash":"[a-f0-9]+"' | cut -d: -f2 | tr -d '"')
HASH_C_FINAL=$(echo "$TIP_C_FINAL" | grep -oE '"hash":"[a-f0-9]+"' | cut -d: -f2 | tr -d '"')

assert_eq "$HASH_C_FINAL" "$HASH_A_FINAL" "Node C and Node A tip hash must match after synchronization"

echo "✅ Chain Synchronization & Catch-Up Successful!"

echo "========================================================="
echo "🎉 ALL MULTI-NODE INTEGRATION TESTS PASSED SUCCESSFULLY!"
echo "========================================================="
exit 0
