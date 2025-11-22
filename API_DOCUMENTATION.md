# DID Explorer API Documentation

**Base URL**: `https://your-did-explorer-api.onrender.com` (or your deployed URL)

This API provides real-time and historical access to DID (Decentralized Identifier) claim events from the Sui blockchain.

---

## 📡 Server-Sent Events (SSE) Endpoint

### Real-time Event Stream

**Endpoint**: `GET /api/sse/events`

**Description**: Subscribe to real-time DID claim events as they are indexed from the blockchain.

**Connection Type**: Server-Sent Events (SSE)

**Headers**:
```
Accept: text/event-stream
Cache-Control: no-cache
Connection: keep-alive
```

### Event Format

Each event is sent as a JSON object with the following structure:

```json
{
  "id": 1,
  "registry_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "user_address": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
  "did_type": "github",
  "user_did_id": "octocat",
  "nft_id": "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba",
  "checkpoint_sequence_number": 264113881,
  "transaction_digest": "5xK8mN9pQ2rS3tU4vW5xY6zA7bC8dE9fG0hI1jK2lM3nO4pQ5rS6tU7vW8xY9zA0",
  "timestamp_ms": 1732136414611,
  "event_index": 0
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Auto-incrementing database ID |
| `registry_id` | string | Sui object ID of the DID registry |
| `user_address` | string | Sui address of the user who claimed the DID |
| `did_type` | string | Type of DID (e.g., "github", "twitter", "discord") |
| `user_did_id` | string | The actual DID identifier (e.g., GitHub username) |
| `nft_id` | string | Sui object ID of the minted NFT |
| `checkpoint_sequence_number` | integer | Blockchain checkpoint number |
| `transaction_digest` | string | Sui transaction hash |
| `timestamp_ms` | integer | Unix timestamp in milliseconds |
| `event_index` | integer | Event index within the transaction |

### Frontend Example (JavaScript)

```javascript
// Connect to SSE endpoint
const eventSource = new EventSource('https://your-api.com/api/sse/events');

// Listen for messages
eventSource.onmessage = (event) => {
  const didEvent = JSON.parse(event.data);
  console.log('New DID claimed:', didEvent);
  
  // Update your UI
  displayNewDIDClaim(didEvent);
};

// Handle errors
eventSource.onerror = (error) => {
  console.error('SSE connection error:', error);
  // Reconnect logic here
};

// Close connection when needed
// eventSource.close();
```

### Frontend Example (React)

```jsx
import { useEffect, useState } from 'react';

function DIDEventStream() {
  const [events, setEvents] = useState([]);

  useEffect(() => {
    const eventSource = new EventSource('https://your-api.com/api/sse/events');

    eventSource.onmessage = (event) => {
      const didEvent = JSON.parse(event.data);
      setEvents(prev => [didEvent, ...prev]); // Add to beginning
    };

    eventSource.onerror = (error) => {
      console.error('SSE Error:', error);
    };

    return () => {
      eventSource.close();
    };
  }, []);

  return (
    <div>
      <h2>Real-time DID Claims</h2>
      {events.map(event => (
        <div key={event.id}>
          <p>{event.user_did_id} claimed {event.did_type} DID</p>
          <small>{new Date(event.timestamp_ms).toLocaleString()}</small>
        </div>
      ))}
    </div>
  );
}
```

---

## 📊 REST API Endpoint

### Get Historical DID Claims (Bulk)

**Endpoint**: `GET /api/events`

**Description**: Retrieve historical DID claim events with pagination and filtering.

**Method**: `GET`

**Query Parameters**:

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `limit` | integer | No | 100 | Number of events to return (max: 1000) |
| `offset` | integer | No | 0 | Number of events to skip (for pagination) |
| `did_type` | string | No | - | Filter by DID type (e.g., "github", "twitter") |
| `user_address` | string | No | - | Filter by user's Sui address |
| `from_timestamp` | integer | No | - | Filter events after this timestamp (ms) |
| `to_timestamp` | integer | No | - | Filter events before this timestamp (ms) |

### Response Format

**Success Response** (200 OK):

```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "registry_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
      "user_address": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "did_type": "github",
      "user_did_id": "octocat",
      "nft_id": "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba",
      "checkpoint_sequence_number": 264113881,
      "transaction_digest": "5xK8mN9pQ2rS3tU4vW5xY6zA7bC8dE9fG0hI1jK2lM3nO4pQ5rS6tU7vW8xY9zA0",
      "timestamp_ms": 1732136414611,
      "event_index": 0
    },
    {
      "id": 2,
      "registry_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
      "user_address": "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
      "did_type": "twitter",
      "user_did_id": "elonmusk",
      "nft_id": "0x1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff",
      "checkpoint_sequence_number": 264113882,
      "transaction_digest": "6yL9nO0qR3sT4uV5wX6yZ7aB8cD9eF0gH1iJ2kL3mN4oP5qR6sT7uV8wX9yZ0aB1",
      "timestamp_ms": 1732136415622,
      "event_index": 1
    }
  ],
  "pagination": {
    "limit": 100,
    "offset": 0,
    "total": 2,
    "has_more": false
  }
}
```

### Frontend Example (JavaScript)

```javascript
// Fetch historical events
async function fetchDIDEvents(options = {}) {
  const params = new URLSearchParams({
    limit: options.limit || 100,
    offset: options.offset || 0,
    ...(options.did_type && { did_type: options.did_type }),
    ...(options.user_address && { user_address: options.user_address }),
  });

  const response = await fetch(`https://your-api.com/api/events?${params}`);
  const data = await response.json();
  
  return data;
}

// Usage
const events = await fetchDIDEvents({ 
  limit: 50, 
  did_type: 'github' 
});

console.log(`Found ${events.pagination.total} events`);
events.data.forEach(event => {
  console.log(`${event.user_did_id} claimed ${event.did_type} DID`);
});
```

### Frontend Example (React with Pagination)

```jsx
import { useEffect, useState } from 'react';

function DIDEventHistory() {
  const [events, setEvents] = useState([]);
  const [loading, setLoading] = useState(true);
  const [pagination, setPagination] = useState({ offset: 0, limit: 20 });

  const fetchEvents = async () => {
    setLoading(true);
    try {
      const response = await fetch(
        `https://your-api.com/api/events?limit=${pagination.limit}&offset=${pagination.offset}`
      );
      const data = await response.json();
      
      setEvents(data.data);
      setPagination(prev => ({ ...prev, ...data.pagination }));
    } catch (error) {
      console.error('Failed to fetch events:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchEvents();
  }, [pagination.offset]);

  const nextPage = () => {
    setPagination(prev => ({ ...prev, offset: prev.offset + prev.limit }));
  };

  const prevPage = () => {
    setPagination(prev => ({ 
      ...prev, 
      offset: Math.max(0, prev.offset - prev.limit) 
    }));
  };

  if (loading) return <div>Loading...</div>;

  return (
    <div>
      <h2>DID Claim History</h2>
      <table>
        <thead>
          <tr>
            <th>DID Type</th>
            <th>Username</th>
            <th>User Address</th>
            <th>Timestamp</th>
          </tr>
        </thead>
        <tbody>
          {events.map(event => (
            <tr key={event.id}>
              <td>{event.did_type}</td>
              <td>{event.user_did_id}</td>
              <td>{event.user_address.slice(0, 10)}...</td>
              <td>{new Date(event.timestamp_ms).toLocaleString()}</td>
            </tr>
          ))}
        </tbody>
      </table>
      
      <div>
        <button onClick={prevPage} disabled={pagination.offset === 0}>
          Previous
        </button>
        <span>
          Showing {pagination.offset + 1} - {pagination.offset + events.length} of {pagination.total}
        </span>
        <button onClick={nextPage} disabled={!pagination.has_more}>
          Next
        </button>
      </div>
    </div>
  );
}
```

---

## 🔍 Example Use Cases

### 1. Real-time Notification System

```javascript
const eventSource = new EventSource('https://your-api.com/api/sse/events');

eventSource.onmessage = (event) => {
  const didEvent = JSON.parse(event.data);
  
  // Show browser notification
  if (Notification.permission === 'granted') {
    new Notification('New DID Claimed!', {
      body: `${didEvent.user_did_id} claimed a ${didEvent.did_type} DID`,
      icon: '/did-icon.png'
    });
  }
};
```

### 2. User Profile Page

```javascript
// Fetch all DIDs claimed by a specific user
async function getUserDIDs(userAddress) {
  const response = await fetch(
    `https://your-api.com/api/events?user_address=${userAddress}&limit=1000`
  );
  const data = await response.json();
  
  return data.data.map(event => ({
    type: event.did_type,
    username: event.user_did_id,
    nftId: event.nft_id,
    claimedAt: new Date(event.timestamp_ms)
  }));
}
```

### 3. Statistics Dashboard

```javascript
// Fetch recent events and calculate statistics
async function getDIDStats() {
  const response = await fetch('https://your-api.com/api/events?limit=1000');
  const data = await response.json();
  
  const stats = {
    total: data.pagination.total,
    byType: {},
    recentClaims: data.data.slice(0, 10)
  };
  
  data.data.forEach(event => {
    stats.byType[event.did_type] = (stats.byType[event.did_type] || 0) + 1;
  });
  
  return stats;
}
```

---

## ⚠️ Error Handling

### SSE Connection Errors

```javascript
const eventSource = new EventSource('https://your-api.com/api/sse/events');

eventSource.onerror = (error) => {
  console.error('SSE Error:', error);
  
  // Automatic reconnection is built-in, but you can add custom logic
  if (eventSource.readyState === EventSource.CLOSED) {
    console.log('Connection closed, will retry...');
  }
};
```

### REST API Errors

**Error Response** (4xx/5xx):

```json
{
  "success": false,
  "error": {
    "code": "INVALID_PARAMETER",
    "message": "Invalid limit parameter. Must be between 1 and 1000."
  }
}
```

---

## 🚀 Rate Limiting

- **SSE Endpoint**: No rate limit (single persistent connection)
- **REST API**: 100 requests per minute per IP address

---

## 📝 Notes for Frontend Developers

1. **SSE Connection Management**:
   - The browser automatically reconnects if the connection drops
   - Consider adding a visual indicator for connection status
   - Handle the `onerror` event for better UX

2. **Data Freshness**:
   - SSE provides real-time updates (typically within 1-2 seconds of blockchain confirmation)
   - REST API returns data from the database (updated in real-time by the indexer)

3. **Timestamps**:
   - All timestamps are in milliseconds (Unix epoch)
   - Convert to JavaScript Date: `new Date(timestamp_ms)`

4. **Sui Addresses**:
   - All addresses are 66 characters (including `0x` prefix)
   - Consider truncating for display: `address.slice(0, 10) + '...'`

5. **Pagination**:
   - Always check `has_more` to determine if there are more pages
   - Use `offset` and `limit` for pagination
   - Maximum `limit` is 1000 events per request

6. **Filtering**:
   - Combine multiple query parameters for complex filtering
   - All filters are case-sensitive
   - Use URL encoding for special characters

---

## 🔗 Additional Resources

- **Sui Explorer**: https://suiscan.xyz/testnet/home
- **Transaction Details**: `https://suiscan.xyz/testnet/tx/{transaction_digest}`
- **Object Details**: `https://suiscan.xyz/testnet/object/{nft_id}`

---

## 📞 Support

For API issues or questions, please contact the backend team or create an issue in the repository.
