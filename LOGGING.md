# 📊 Logging Configuration Guide

## Overview

The SuiVerify Indexer includes comprehensive, configurable logging to help monitor event processing in both development and production environments. You can control logging behavior through environment variables.

## 🔧 Configuration

### Environment Variables

Configure logging through your `.env` file:

```bash
# Enable/disable detailed logging
ENABLE_DETAILED_LOGS=true

# Set log level (trace, debug, info, warn, error)
LOG_LEVEL=info

# Enable specific event processing logs
LOG_EVENTS=true
```

### Production vs Development Settings

**Development (Verbose Logging):**
```bash
ENABLE_DETAILED_LOGS=true
LOG_LEVEL=debug
LOG_EVENTS=true
```

**Production (Minimal Logging):**
```bash
ENABLE_DETAILED_LOGS=false
LOG_LEVEL=warn
LOG_EVENTS=false
```

## 📝 Log Types

### 1. Startup Logs
When `ENABLE_DETAILED_LOGS=true`:
```
🚀 Starting SuiVerify Indexer with detailed logging enabled
📊 Log configuration: LogConfig { enable_detailed_logs: true, log_level: "info", log_events: true }
✅ Registered TransactionDigestHandler pipeline
✅ Registered DIDClaimedEventHandler pipeline
🎯 Monitoring events for package: 0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09
```

### 2. Event Processing Logs
When `LOG_EVENTS=true`:

**Event Discovery:**
```
🎯 Found DIDClaimed event in tx: a1b2c3d4 at index: 0
```

**Event Details:**
```
📝 DIDClaimed Event Details:
   Registry ID: 0x123...
   User Address: 0xabc...
   DID Type: 1
   User DID ID: 0xdef...
   NFT ID: 0x456...
```

**Processing Summary:**
```
✅ Processed 3 DIDClaimed events from checkpoint 12345
```

### 3. Database Operations
When `ENABLE_DETAILED_LOGS=true`:

**Batch Processing:**
```
💾 Committing 5 DIDClaimed events to database
💾 Successfully inserted 5 new DIDClaimed events
```

**Duplicate Handling:**
```
💾 No new events inserted (duplicates skipped)
```

### 4. Checkpoint Processing
When `ENABLE_DETAILED_LOGS=true`:
```
🔍 Processing checkpoint 12345 with 150 transactions
```

### 5. Error Handling
```
⚠️  Failed to deserialize DIDClaimed event in tx a1b2c3d4: Invalid BCS format
```

## 🎛️ Log Levels

| Level | Description | Use Case |
|-------|-------------|----------|
| `trace` | Most verbose, includes all details | Deep debugging |
| `debug` | Detailed information for debugging | Development |
| `info` | General information about operations | Development/Staging |
| `warn` | Warning messages about potential issues | Production |
| `error` | Error messages only | Production (minimal) |

## 🚀 Usage Examples

### Development Setup
```bash
# .env file for development
ENABLE_DETAILED_LOGS=true
LOG_LEVEL=info
LOG_EVENTS=true

# Run indexer
cargo run --release -- --remote-store-url https://checkpoints.testnet.sui.io
```

**Expected Output:**
```
🚀 Starting SuiVerify Indexer with detailed logging enabled
📊 Log configuration: LogConfig { enable_detailed_logs: true, log_level: "info", log_events: true }
✅ Registered TransactionDigestHandler pipeline
✅ Registered DIDClaimedEventHandler pipeline
🎯 Monitoring events for package: 0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09
🔍 Processing checkpoint 1000 with 25 transactions
🎯 Found DIDClaimed event in tx: a1b2c3d4 at index: 0
📝 DIDClaimed Event Details:
   Registry ID: 0x123...
   User Address: 0xabc...
   DID Type: 1
   User DID ID: 0xdef...
   NFT ID: 0x456...
✅ Processed 1 DIDClaimed events from checkpoint 1000
💾 Successfully inserted 1 new DIDClaimed events
```

### Production Setup
```bash
# .env file for production
ENABLE_DETAILED_LOGS=false
LOG_LEVEL=warn
LOG_EVENTS=false

# Run indexer
cargo run --release -- --remote-store-url https://checkpoints.mainnet.sui.io
```

**Expected Output (Minimal):**
```
# Only warnings and errors will be shown
⚠️  Failed to deserialize DIDClaimed event in tx a1b2c3d4: Invalid BCS format
```

## 🔍 Monitoring & Debugging

### Finding Issues
1. **Event Not Found**: Check if `LOG_EVENTS=true` and look for "Found DIDClaimed event" messages
2. **Deserialization Errors**: Look for warning messages with "Failed to deserialize"
3. **Database Issues**: Check for database commit logs when `ENABLE_DETAILED_LOGS=true`

### Performance Monitoring
- Monitor checkpoint processing frequency
- Track event insertion rates
- Watch for duplicate event handling

### Log Filtering
Use standard log filtering tools:
```bash
# Filter for event-specific logs
cargo run 2>&1 | grep "DIDClaimed"

# Filter for errors only
cargo run 2>&1 | grep "⚠️\|❌"

# Filter for successful operations
cargo run 2>&1 | grep "✅"
```

## 🏗️ Implementation Details

### Log Configuration Structure
```rust
pub struct LogConfig {
    pub enable_detailed_logs: bool,  // Master switch for detailed logging
    pub log_level: String,           // Global log level
    pub log_events: bool,           // Event-specific logging
}
```

### Conditional Logging
```rust
// Only log if detailed logging is enabled
if self.log_config.should_log_detailed() {
    debug!("🔍 Processing checkpoint {}", checkpoint_seq);
}

// Only log events if both detailed and event logging are enabled
if self.log_config.should_log_events() {
    info!("🎯 Found DIDClaimed event in tx: {}", tx_digest);
}
```

## 📈 Best Practices

1. **Development**: Use verbose logging to understand event flow
2. **Staging**: Use moderate logging to catch issues before production
3. **Production**: Use minimal logging to reduce overhead and noise
4. **Monitoring**: Set up log aggregation for production deployments
5. **Alerts**: Monitor for error patterns and processing delays

## 🔄 Dynamic Configuration

To change logging without restart:
1. Update `.env` file
2. Send SIGHUP to the process (if implemented)
3. Or restart the indexer service

## 📊 Log Analysis

### Useful Patterns
```bash
# Count events processed per hour
grep "Successfully inserted" logs.txt | grep "$(date +%Y-%m-%d\ %H)" | wc -l

# Find failed deserializations
grep "Failed to deserialize" logs.txt

# Monitor checkpoint processing rate
grep "Processing checkpoint" logs.txt | tail -10
```

This logging system provides the flexibility you need for both development debugging and production monitoring while keeping performance impact minimal when logging is disabled.
