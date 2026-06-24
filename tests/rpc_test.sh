#!/usr/bin/env bash
# Integration test for ARUNA RPC Query APIs

set -euo pipefail

echo "=== Starting ARUNA RPC Integration Tests ==="

# Clean up any old database directory for port 9002
rm -rf ./data_sumatera_9002

# Start node in the background on custom ports to prevent conflicts
# P2P port 9002, RPC port 8082
echo "Starting aruna-node..."
./target/debug/aruna-node --p2p-port 9002 --rpc-port 8082 > node_test.log 2>&1 &
NODE_PID=$!

# Ensure we kill the node on exit
cleanup() {
    echo "Cleaning up..."
    kill $NODE_PID || true
    rm -rf ./data_sumatera_9002
    echo "Done."
}
trap cleanup EXIT

# Wait for node to start
echo "Waiting for RPC server to start on 127.0.0.1:8082..."
for i in {1..10}; do
    if curl -s http://127.0.0.1:8082/status > /dev/null; then
        echo "Node RPC server is up!"
        break
    fi
    sleep 1
done

# If node didn't start, print logs and exit
if ! curl -s http://127.0.0.1:8082/status > /dev/null; then
    echo "ERROR: Node failed to start. Logs:"
    cat node_test.log
    exit 1
fi

# Helper to assert string contains a pattern
assert_contains() {
    local data="$1"
    local pattern="$2"
    local msg="$3"
    if [[ "$data" != *"$pattern"* ]]; then
        echo "FAIL: $msg"
        echo "Expected pattern: $pattern"
        echo "Received data: $data"
        exit 1
    fi
}

# 1. Test GET /status
echo "Testing GET /status..."
STATUS_RES=$(curl -s http://127.0.0.1:8082/status)
echo "Response: $STATUS_RES"
assert_contains "$STATUS_RES" "sumatera" "GET /status must return network 'sumatera'"
assert_contains "$STATUS_RES" "height" "GET /status must return height"

# 2. Test GET /chain/tip
echo "Testing GET /chain/tip..."
TIP_RES=$(curl -s http://127.0.0.1:8082/chain/tip)
echo "Response: $TIP_RES"
assert_contains "$TIP_RES" "hash" "GET /chain/tip must return best block hash"
assert_contains "$TIP_RES" "height" "GET /chain/tip must return best block height"

# 3. Test GET /blocks
echo "Testing GET /blocks..."
BLOCKS_RES=$(curl -s http://127.0.0.1:8082/blocks)
echo "Response: $BLOCKS_RES"
assert_contains "$BLOCKS_RES" "height" "GET /blocks must return block height"
assert_contains "$BLOCKS_RES" "hash" "GET /blocks must return block hash"
assert_contains "$BLOCKS_RES" "tx_count" "GET /blocks must return tx_count"

# 4. Test GET /block/0 (Genesis)
echo "Testing GET /block/0..."
BLOCK0_RES=$(curl -s http://127.0.0.1:8082/block/0)
echo "Response: $BLOCK0_RES"
assert_contains "$BLOCK0_RES" "hash" "GET /block/0 must return block hash"
assert_contains "$BLOCK0_RES" "header" "GET /block/0 must return block header"
assert_contains "$BLOCK0_RES" "body" "GET /block/0 must return block body"

# 5. Test GET /address/{address}
# Pre-funded address 1: sum1qyqszqgpqyqszqgpqyqszqgpqyqszqgpe6sslr (5,000,000 ARU)
echo "Testing GET /address/sum1qyqszqgpqyqszqgpqyqszqgpqyqszqgpe6sslr..."
ADDR1_RES=$(curl -s http://127.0.0.1:8082/address/sum1qyqszqgpqyqszqgpqyqszqgpqyqszqgpe6sslr)
echo "Response: $ADDR1_RES"
assert_contains "$ADDR1_RES" "5000000000000" "GET /address must return correct balance for pre-funded account"
assert_contains "$ADDR1_RES" "nonce" "GET /address must return nonce"

# Pre-funded address 2: sum1lmmdlfytquujf3pk2wgpp4up97l9qzt2k5tk4d (10 ARU = 10,000,000 micro-ARU)
echo "Testing GET /address/sum1lmmdlfytquujf3pk2wgpp4up97l9qzt2k5tk4d..."
ADDR2_RES=$(curl -s http://127.0.0.1:8082/address/sum1lmmdlfytquujf3pk2wgpp4up97l9qzt2k5tk4d)
echo "Response: $ADDR2_RES"
assert_contains "$ADDR2_RES" "10000000" "GET /address must return correct balance (10,000,000 micro-ARU)"

# Uninitialized address (should return default balance=15,000,000 ARU)
echo "Testing GET /address/sum1q5zs2pg9q5zs2pg9q5zs2pg9q5zs2pg9gw3snf..."
ADDR3_RES=$(curl -s http://127.0.0.1:8082/address/sum1q5zs2pg9q5zs2pg9q5zs2pg9q5zs2pg9gw3snf)
echo "Response: $ADDR3_RES"
assert_contains "$ADDR3_RES" "15000000000000" "GET /address must return correct balance (15,000,000 ARU)"

# Invalid address (wrong prefix/HRP)
echo "Testing GET /address/kal1qyqszqgpqyqszqgpqyqszqgpqyqszqgpe6sslr..."
ADDR_INVALID_RES=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8082/address/kal1qyqszqgpqyqszqgpqyqszqgpqyqszqgpe6sslr || true)
echo "HTTP Status Code: $ADDR_INVALID_RES"
if [ "$ADDR_INVALID_RES" != "400" ]; then
    echo "FAIL: Expected 400 Bad Request for wrong address prefix, got $ADDR_INVALID_RES"
    exit 1
fi

# 6. Test Transaction Submission & GET /transaction/{hash}
echo "Submitting a test transaction..."
TX_SUBMIT_RES=$(curl -s -X POST -H "Content-Type: application/json" -d @test_tx.json http://127.0.0.1:8082/tx)
echo "Response: $TX_SUBMIT_RES"
assert_contains "$TX_SUBMIT_RES" "success" "Transaction submission must succeed"

# Extract tx hash
TX_HASH=$(echo "$TX_SUBMIT_RES" | grep -oE '[a-f0-9]{64}')
echo "Submitted Tx Hash: $TX_HASH"

# Test GET /transaction/{hash} for pending transaction
echo "Testing GET /transaction/$TX_HASH (pending)..."
TX_RES=$(curl -s http://127.0.0.1:8082/transaction/$TX_HASH)
echo "Response: $TX_RES"
assert_contains "$TX_RES" "pending" "Transaction must be in 'pending' status"
assert_contains "$TX_RES" "transaction" "Transaction response must include envelope details"

# Test GET /transaction/{hash} for non-existent transaction
echo "Testing GET /transaction/0000000000000000000000000000000000000000000000000000000000000000 (not found)..."
TX_NOT_FOUND_RES=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8082/transaction/0000000000000000000000000000000000000000000000000000000000000000 || true)
echo "HTTP Status Code: $TX_NOT_FOUND_RES"
if [ "$TX_NOT_FOUND_RES" != "404" ]; then
    echo "FAIL: Expected 404 Not Found for missing transaction, got $TX_NOT_FOUND_RES"
    exit 1
fi

echo "=== All RPC Query API Integration Tests Passed! ==="
exit 0
