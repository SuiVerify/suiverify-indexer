# Complete Flow Testing Guide

## Architecture Overview

```
Sui Blockchain → Indexer → PostgreSQL (permanent storage)
                        ↓
                    Redis Pub/Sub (ephemeral messaging)
                        ↓
                    SSE Server (did-explorer)
                        ↓
                    Frontend/Clients (via SSE or REST)
```

## Manual Testing Steps

### Prerequisites

1. **PostgreSQL** running with `sui_indexer` database
2. **Redis** accessible (Cloud Redis or local)
3. **Indexer** built (`suiverify-indexer`)
4. **SSE Server** built (`did-explorer`)

### Test 1: Monitor Redis Pub/Sub

**Terminal 1** - Subscribe to Redis channel:
```bash
redis-cli -h redis-11134.crce182.ap-south-1-1.ec2.cloud.redislabs.com \
          -p 11134 \
          -a YOUR_PASSWORD \
          SUBSCRIBE did_claimed
```

You should see:
```
Reading messages... (press Ctrl-C to quit)
1) "subscribe"
2) "did_claimed"
3) (integer) 1
```

### Test 2: Start SSE Server

**Terminal 2** - Run the SSE API server:
```bash
cd /home/ash-win/projects/suiverify/did-explorer
RUST_LOG=info cargo run --release
```

Expected output:
```
🚀 Starting SSE API Server
✅ Connected to PostgreSQL
🔌 Connecting to Redis...
✅ Subscribed to Redis channel: did_claimed
👂 Listening for DID claim events from Redis...
✅ API server listening on http://127.0.0.1:8080
📡 SSE endpoint: http://127.0.0.1:8080/api/sse/events
🔍 REST endpoint: http://127.0.0.1:8080/api/events
```

### Test 3: Connect SSE Client

**Terminal 3** - Listen for real-time events via SSE:
```bash
curl -N http://localhost:8080/api/sse/events
```

This will keep the connection open and wait for events. You should see:
```
(waiting for events...)
```

### Test 4: Trigger Real Events

**Terminal 4** - Run the indexer:

First, reset the watermark to reprocess the test checkpoint:
```bash
psql -d sui_indexer -c "DELETE FROM watermarks WHERE pipeline = 'did_claimed_event_handler';"
psql -d sui_indexer -c "TRUNCATE TABLE did_claimed_events;"
```

Then run the indexer:
```bash
cd /home/ash-win/projects/suiverify/suiverify-indexer
RUST_LOG=info cargo run --release -- \
  --remote-store-url https://checkpoints.testnet.sui.io \
  --first-checkpoint 264113880 \
  --last-checkpoint 264113882
```

### Expected Flow

1. **Terminal 4 (Indexer)** will show:
   ```
   Found DIDClaimed event in tx: GBeCguCs at index: 0
   ✅ Successfully inserted 1 new DIDClaimed events to PostgreSQL
   📤 Published event to Redis Pub/Sub channel 'did_claimed'
   ```

2. **Terminal 1 (Redis Monitor)** will show:
   ```
   1) "message"
   2) "did_claimed"
   3) "{\"registry_id\":\"0x8587978d...\",\"user_address\":\"0xcca6db49...\",...}"
   ```

3. **Terminal 2 (SSE Server)** will show:
   ```
   📬 Received event from Redis Pub/Sub
   📤 Broadcasting to 1 SSE clients
   ✅ Pushed to 1 client(s)
   ```

4. **Terminal 3 (SSE Client)** will show:
   ```
   data: {"registry_id":"0x8587978d0bc2d856f8366d57e797cbe1419bab99a43265b1df97e6d5d27f3300","user_address":"0xcca6db49f975b25b2f98d76db7f505b487bcfd9eeeadfea06b51e2fe126fb9e4","did_type":1,"user_did_id":"0xb707d35dd0ab88a7d37e85fc9285d7d00bdb26abd536bac7e0fd3567f3376dfc","nft_id":"0xc6a42a098f3b122e63eaab8a92f50396f11b308ecff011a1e603397f11cc5d0f","checkpoint":264113881,"tx_digest":"GBeCguCsY9HVQs2qXhV7FjgKauLWG5pjaYt7gy2ksSde","timestamp":1763136414389}
   ```

### Test 5: Query Historical Events (REST API)

In any terminal:
```bash
# Get all recent events
curl http://localhost:8080/api/events

# Get events with limit
curl "http://localhost:8080/api/events?limit=5"

# Get events for specific user
curl "http://localhost:8080/api/events?user_address=0xcca6db49f975b25b2f98d76db7f505b487bcfd9eeeadfea06b51e2fe126fb9e4"
```

Expected response:
```json
[
  {
    "registry_id": "0x8587978d0bc2d856f8366d57e797cbe1419bab99a43265b1df97e6d5d27f3300",
    "user_address": "0xcca6db49f975b25b2f98d76db7f505b487bcfd9eeeadfea06b51e2fe126fb9e4",
    "did_type": 1,
    "user_did_id": "0xb707d35dd0ab88a7d37e85fc9285d7d00bdb26abd536bac7e0fd3567f3376dfc",
    "nft_id": "0xc6a42a098f3b122e63eaab8a92f50396f11b308ecff011a1e603397f11cc5d0f",
    "checkpoint": 264113881,
    "tx_digest": "GBeCguCsY9HVQs2qXhV7FjgKauLWG5pjaYt7gy2ksSde",
    "timestamp": 1763136414389
  }
]
```

### Test 6: Manual Redis Publish (Optional)

Test the pipeline without running the indexer:

```bash
redis-cli -h redis-11134.crce182.ap-south-1-1.ec2.cloud.redislabs.com \
          -p 11134 \
          -a YOUR_PASSWORD \
          PUBLISH did_claimed '{"registry_id":"0xTEST","user_address":"0xUSER","did_type":1,"user_did_id":"0xDID","nft_id":"0xNFT","checkpoint":123,"tx_digest":"TEST","timestamp":1234567890}'
```

The SSE client (Terminal 3) should immediately receive this test event.

## Verification Checklist

- [ ] Redis connection successful
- [ ] PostgreSQL connection successful  
- [ ] SSE Server starts without errors
- [ ] SSE client connects successfully
- [ ] Indexer finds and processes DIDClaimed event
- [ ] Event stored in PostgreSQL
- [ ] Event published to Redis
- [ ] SSE Server receives event from Redis
- [ ] SSE client receives event in real-time
- [ ] REST API returns historical events

## Troubleshooting

### SSE Server won't start
- Check if port 8080 is already in use: `lsof -i :8080`
- Check Redis URL in `.env` file
- Check PostgreSQL connection

### No events in SSE client
- Verify SSE Server is subscribed to Redis (check Terminal 2 logs)
- Verify indexer published to Redis (check Terminal 4 logs)
- Try manual Redis publish test

### Events not in database
- Check indexer logs for errors
- Verify watermark was reset
- Check PostgreSQL permissions

## Production Deployment

For production, run the indexer continuously:

```bash
cd /home/ash-win/projects/suiverify/suiverify-indexer
RUST_LOG=info cargo run --release -- \
  --remote-store-url https://checkpoints.testnet.sui.io
```

The indexer will:
- Resume from the last processed checkpoint (watermark)
- Continuously process new checkpoints as they arrive
- Store events in PostgreSQL
- Publish to Redis for real-time notifications

## Frontend Integration

### React/Next.js Example

```javascript
// On page load - fetch historical events
useEffect(() => {
  async function fetchEvents() {
    const response = await fetch('http://localhost:8080/api/events?limit=20');
    const events = await response.json();
    setEvents(events);
  }
  fetchEvents();
}, []);

// Subscribe to real-time updates
useEffect(() => {
  const eventSource = new EventSource('http://localhost:8080/api/sse/events');
  
  eventSource.onmessage = (e) => {
    const newEvent = JSON.parse(e.data);
    setEvents(prev => [newEvent, ...prev]);
    // Show notification
    toast.success(`New DID claimed: ${newEvent.user_address.slice(0, 8)}...`);
  };
  
  eventSource.onerror = () => {
    console.error('SSE connection error');
    eventSource.close();
  };
  
  return () => eventSource.close();
}, []);
```

## Key Concepts

### Redis Pub/Sub Behavior

- **Fire-and-forget**: If no subscriber exists, messages are dropped
- **No storage**: Messages are not persisted
- **Instant delivery**: <1ms latency when subscriber exists
- **This is OK**: PostgreSQL has all data as source of truth

### Why This Architecture?

✅ **PostgreSQL**: Permanent storage, source of truth  
✅ **Redis Pub/Sub**: Real-time notifications, lightweight  
✅ **SSE**: Simple one-way push, better than WebSocket for this use case  
✅ **REST API**: Historical queries, user dashboards  

❌ **NOT using**:
- Redis Streams (unnecessary complexity)
- WebSocket (overkill for one-way push)
- PostgreSQL LISTEN/NOTIFY (complex in async Rust)
