#!/bin/bash

# Complete Flow Test for SuiVerify Indexer → Redis → SSE Server
# This script tests the entire real-time event streaming pipeline

echo "🚀 SuiVerify Event Streaming Test"
echo "=================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Redis is accessible
echo -e "${BLUE}Step 1: Testing Redis connection...${NC}"
if redis-cli -h redis-11134.crce182.ap-south-1-1.ec2.cloud.redislabs.com -p 11134 -a YOUR_PASSWORD PING > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Redis connection successful${NC}"
else
    echo -e "${YELLOW}⚠️  Redis connection failed. Please update YOUR_PASSWORD in this script.${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}Step 2: Testing PostgreSQL connection...${NC}"
if psql -d sui_indexer -c "SELECT COUNT(*) FROM did_claimed_events;" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PostgreSQL connection successful${NC}"
    EVENT_COUNT=$(psql -d sui_indexer -t -c "SELECT COUNT(*) FROM did_claimed_events;")
    echo "   Current events in database: $EVENT_COUNT"
else
    echo -e "${YELLOW}⚠️  PostgreSQL connection failed${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}Step 3: Starting SSE Server (did-explorer)...${NC}"
echo "   This will run in the background"
cd /home/ash-win/projects/suiverify/did-explorer
cargo run --release > /tmp/sse-server.log 2>&1 &
SSE_PID=$!
echo "   SSE Server PID: $SSE_PID"
sleep 5

# Check if SSE server started successfully
if ps -p $SSE_PID > /dev/null; then
    echo -e "${GREEN}✅ SSE Server started successfully${NC}"
    echo "   Logs: /tmp/sse-server.log"
else
    echo -e "${YELLOW}⚠️  SSE Server failed to start${NC}"
    cat /tmp/sse-server.log
    exit 1
fi

echo ""
echo -e "${BLUE}Step 4: Testing SSE endpoint...${NC}"
echo "   Connecting to http://localhost:8080/api/sse/events"
echo "   (This will listen for real-time events)"
echo ""
echo -e "${YELLOW}   Press Ctrl+C to stop listening${NC}"
echo ""

# Start SSE listener in background
curl -N http://localhost:8080/api/sse/events &
CURL_PID=$!

echo ""
echo -e "${BLUE}Step 5: Ready to test!${NC}"
echo ""
echo "Now you can:"
echo "  1. Run the indexer to trigger real events:"
echo "     cd /home/ash-win/projects/suiverify/suiverify-indexer"
echo "     RUST_LOG=info cargo run --release -- --remote-store-url https://checkpoints.testnet.sui.io --first-checkpoint 264113880 --last-checkpoint 264113882"
echo ""
echo "  2. Or manually publish a test event to Redis:"
echo "     redis-cli -h redis-11134.crce182.ap-south-1-1.ec2.cloud.redislabs.com -p 11134 -a YOUR_PASSWORD PUBLISH did_claimed '{\"registry_id\":\"0xTEST\",\"user_address\":\"0xUSER\",\"did_type\":1,\"user_did_id\":\"0xDID\",\"nft_id\":\"0xNFT\",\"checkpoint\":123,\"tx_digest\":\"TEST\",\"timestamp\":1234567890}'"
echo ""
echo "  3. Query historical events via REST API:"
echo "     curl http://localhost:8080/api/events"
echo ""

# Wait for user to press Ctrl+C
trap "echo ''; echo 'Cleaning up...'; kill $SSE_PID $CURL_PID 2>/dev/null; exit 0" INT

wait
