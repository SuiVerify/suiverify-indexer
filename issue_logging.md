# Issue Log: Sui Indexer Event Storage

## Issue #1: DIDClaimed Events Not Being Stored in Database

**Date**: 2025-11-20  
**Status**: ✅ RESOLVED  
**Severity**: High  

### Problem Description

The `suiverify-indexer` was running successfully and processing checkpoints, but `DIDClaimed` events were not being stored in the `did_claimed_events` PostgreSQL table. The `transaction_digests` table was being populated correctly, indicating the basic indexer pipeline was functional, but the custom event handler was failing silently.

### Symptoms

1. ✅ Indexer running without errors
2. ✅ `transaction_digests` table being populated (218,834+ records)
3. ❌ `did_claimed_events` table remained empty
4. ❌ No "Found DIDClaimed event" logs appearing
5. ❌ Event type matching was failing

### Investigation Process

1. **Initial Hypothesis**: Package ID mismatch
   - Checked the deployed package ID from `current_tx.md`: `0x0d9f5cd6845d838653bac950697ab33009db0a7f886b201dbda9ba132c3dd495`
   - Updated `SUIVERIFY_PACKAGE_ID` constant to match

2. **Debug Logging**: Added logging to inspect actual event types at checkpoint 264113881
   ```rust
   if checkpoint_seq == 264113881 {
       info!("DEBUG: Checkpoint {} - Event type: {}", checkpoint_seq, event_type);
       info!("DEBUG: Expected type: {}", expected_type);
   }
   ```

3. **Discovery**: Found the actual event type in logs:
   ```
   Event type: 0xd9f5cd6845d838653bac950697ab33009db0a7f886b201dbda9ba132c3dd495::did_registry::DIDClaimed
   Expected type: 0x0d9f5cd6845d838653bac950697ab33009db0a7f886b201dbda9ba132c3dd495::did_registry::DIDClaimed
   ```

### Root Cause

**Sui normalizes addresses by removing leading zeros in event type strings.**

Even though the package was published with ID:
```
0x0d9f5cd6845d838653bac950697ab33009db0a7f886b201dbda9ba132c3dd495
```

The event type in the blockchain uses the normalized version:
```
0xd9f5cd6845d838653bac950697ab33009db0a7f886b201dbda9ba132c3dd495
```

This caused the event type matching to fail:
```rust
let expected_type = format!("{}::did_registry::DIDClaimed", SUIVERIFY_PACKAGE_ID);
if event_type == expected_type {
    // This never matched!
}
```

### Solution

Updated `SUIVERIFY_PACKAGE_ID` in `src/event_handlers.rs` to use the normalized address format (without leading zero):

```rust
// Before (INCORRECT)
const SUIVERIFY_PACKAGE_ID: &str = "0x0d9f5cd6845d838653bac950697ab33009db0a7f886b201dbda9ba132c3dd495";

// After (CORRECT)
const SUIVERIFY_PACKAGE_ID: &str = "0xd9f5cd6845d838653bac950697ab33009db0a7f886b201dbda9ba132c3dd495";
```

Also updated the corresponding log message in `src/main.rs`.

### Understanding Watermarks

**Watermarks** are checkpoint tracking records that the indexer framework uses to remember where it left off processing. They act as bookmarks to ensure the indexer can resume from the correct position after restarts.

**Watermark Table Structure**:
```
pipeline                    | checkpoint_hi_inclusive | epoch_hi_inclusive | tx_hi
----------------------------+-------------------------+--------------------+------------
transaction_digest_handler  | 264195913              | 918                | 3047352805
did_claimed_event_handler   | 264113882              | 918                | 3046577002
```

**Key Fields**:
- `pipeline`: Name of the indexer pipeline
- `checkpoint_hi_inclusive`: Highest checkpoint number successfully processed and committed
- `epoch_hi_inclusive`: The epoch number
- `tx_hi`: Highest transaction sequence number processed

**Why We Deleted the Watermark**:

The indexer had previously processed up to checkpoint 264195886, but our test event was at checkpoint 264113881 (earlier). When we tried to run with `--first-checkpoint 264113880`, the indexer ignored it and tried to resume from 264195886:

```
WARN: Ignoring --first-checkpoint and will resume from committer_hi 
pipeline="did_claimed_event_handler" 
first_checkpoint=264113880 
committer_hi=264195886
```

By deleting the watermark, we reset the indexer's memory, allowing it to start fresh from our specified checkpoint.

**When to Delete Watermarks**:
- ✅ Reprocessing old checkpoints for testing
- ✅ Fixing a bug in event handler and reindexing historical data
- ✅ Resetting the indexer to start from scratch
- ❌ **NOT** in production (prevents data loss and duplicate processing)

### Additional Steps Required

After fixing the package ID, we also needed to:

1. **Reset the watermark** to reprocess the checkpoint:
   ```sql
   DELETE FROM watermarks WHERE pipeline = 'did_claimed_event_handler';
   ```

2. **Clear the events table**:
   ```sql
   TRUNCATE TABLE did_claimed_events;
   ```

3. **Run the indexer** with the specific checkpoint range:
   ```bash
   RUST_LOG=info cargo run --release -- \
     --remote-store-url https://checkpoints.testnet.sui.io \
     --first-checkpoint 264113880 \
     --last-checkpoint 264113882
   ```

### Verification

After the fix, the indexer successfully:

1. ✅ Found the `DIDClaimed` event at checkpoint 264113881
2. ✅ Inserted 1 row into the database
3. ✅ Published the event to Redis Pub/Sub channel

**Database Record**:
```
 id |                            registry_id                             |                            user_address                            | did_type |                            user_did_id                             |                               nft_id                               | checkpoint_sequence_number |              transaction_digest              | timestamp_ms  | event_index 
----+--------------------------------------------------------------------+--------------------------------------------------------------------+----------+--------------------------------------------------------------------+--------------------------------------------------------------------+----------------------------+----------------------------------------------+---------------+-------------
  7 | 0x8587978d0bc2d856f8366d57e797cbe1419bab99a43265b1df97e6d5d27f3300 | 0xcca6db49f975b25b2f98d76db7f505b487bcfd9eeeadfea06b51e2fe126fb9e4 |        1 | 0xb707d35dd0ab88a7d37e85fc9285d7d00bdb26abd536bac7e0fd3567f3376dfc | 0xc6a42a098f3b122e63eaab8a92f50396f11b308ecff011a1e603397f11cc5d0f |                  264113881 | GBeCguCsY9HVQs2qXhV7FjgKauLWG5pjaYt7gy2ksSde | 1763136414389 |           0
```

### Key Learnings

1. **Sui Address Normalization**: Always use normalized addresses (without leading zeros) when matching event types, even if the package was published with a leading zero.

2. **Event Type Format**: Sui event types follow the format `PACKAGE_ID::MODULE::EVENT_NAME`, where `PACKAGE_ID` is the normalized address.

3. **Debug Logging**: When debugging event matching issues, log both the actual event type and expected type to identify mismatches.

4. **Watermark Management**: The indexer maintains watermarks to track progress. When reprocessing checkpoints, you may need to delete the watermark for that pipeline.

5. **Clean Build**: In some cases, removing the `target` directory and doing a clean build can help resolve caching issues, though this wasn't the root cause in this case.

### Prevention

To prevent similar issues in the future:

1. **Always normalize package IDs** by removing leading zeros when using them for event type matching
2. **Add validation** to check if the package ID format matches Sui's normalization rules
3. **Include debug logging** in development to verify event type matching
4. **Document** the expected event type format in code comments

### Files Modified

- `src/event_handlers.rs` - Updated `SUIVERIFY_PACKAGE_ID` constant
- `src/main.rs` - Updated package ID in log message

### Related Resources

- [Sui Event Documentation](https://docs.sui.io/concepts/events)
- [Custom Indexing Framework](https://docs.sui.io/guides/developer/advanced/custom-indexer)
- Transaction with DIDClaimed event: `GBeCguCsY9HVQs2qXhV7FjgKauLWG5pjaYt7gy2ksSde`
- Checkpoint: `264113881`

---

## Template for Future Issues

**Date**:  
**Status**: 🔴 OPEN / 🟡 IN PROGRESS / ✅ RESOLVED  
**Severity**: Low / Medium / High / Critical  

### Problem Description

### Symptoms

### Investigation Process

### Root Cause

### Solution

### Verification

### Key Learnings

### Prevention

### Files Modified

### Related Resources
