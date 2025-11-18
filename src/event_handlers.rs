use std::sync::Arc;
use anyhow::Result;
use sui_indexer_alt_framework::{
    pipeline::Processor,
};
use sui_types::full_checkpoint_content::Checkpoint;
use tracing::{info, debug, warn};
use redis::AsyncCommands;

use crate::models::StoredDIDClaimedEvent;
use crate::events::DIDClaimed;
use crate::schema::did_claimed_events::dsl::*;
use crate::config::LogConfig;
use diesel_async::RunQueryDsl;
use sui_indexer_alt_framework::{
    postgres::Db,
    pipeline::sequential::Handler,
    store::Store,
};

// Your package ID from the transaction (updated to match deployed contract - without leading zero)
const SUIVERIFY_PACKAGE_ID: &str = "0xd9f5cd6845d838653bac950697ab33009db0a7f886b201dbda9ba132c3dd495";

pub struct DIDClaimedEventHandler {
    log_config: LogConfig,
}

impl DIDClaimedEventHandler {
    pub fn new(log_config: LogConfig) -> Self {
        Self { log_config }
    }
}

#[async_trait::async_trait]
impl Processor for DIDClaimedEventHandler {
    const NAME: &'static str = "did_claimed_event_handler";

    type Value = StoredDIDClaimedEvent;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<StoredDIDClaimedEvent>> {
        let checkpoint_seq = checkpoint.summary.sequence_number as i64;
        let timestamp = checkpoint.summary.timestamp_ms as i64;
        
        if self.log_config.should_log_detailed() {
            debug!("🔍 Processing checkpoint {} with {} transactions", 
                   checkpoint_seq, checkpoint.transactions.len());
        }
        
        // Log every 1000 checkpoints to show progress
        if checkpoint_seq % 1000 == 0 {
            println!("Processed checkpoint: {}", checkpoint_seq);
        }
        
        let mut events = Vec::new();

        for (_tx_idx, tx) in checkpoint.transactions.iter().enumerate() {
            let tx_digest = tx.transaction.digest().to_string();
            
            // Check if there are events in this transaction
            if let Some(tx_events) = &tx.events {
                // Iterate through all events in this transaction
                for (event_idx, event) in tx_events.data.iter().enumerate() {
                    // Check if this event is from our package and module
                    let event_type = event.type_.to_string();
                    
                    // Format: PACKAGE_ID::MODULE::EVENT_NAME (compiled module name)
                    let expected_type = format!("{}::did_registry::DIDClaimed", SUIVERIFY_PACKAGE_ID);
                    
                    if event_type == expected_type {
                        if self.log_config.should_log_events() {
                            info!("Found DIDClaimed event in tx: {} at index: {}", 
                                  &tx_digest[..8], event_idx);
                        }
                        
                        // Deserialize the event from BCS bytes
                        match bcs::from_bytes::<DIDClaimed>(&event.contents) {
                        Ok(did_claimed) => {
                            if self.log_config.should_log_events() {
                                info!("   DIDClaimed Event Details:");
                                info!("   Registry ID: {}", did_claimed.registry_id);
                                info!("   User Address: {}", did_claimed.user_address);
                                info!("   DID Type: {}", did_claimed.did_type);
                                info!("   User DID ID: {}", did_claimed.user_did_id);
                                info!("   NFT ID: {}", did_claimed.nft_id);
                            }
                            events.push(StoredDIDClaimedEvent {
                                registry_id: did_claimed.registry_id.to_string(),
                                user_address: did_claimed.user_address.to_string(),
                                did_type: did_claimed.did_type as i16,
                                user_did_id: did_claimed.user_did_id.to_string(),
                                nft_id: did_claimed.nft_id.to_string(),
                                checkpoint_sequence_number: checkpoint_seq,
                                transaction_digest: tx_digest.clone(),
                                timestamp_ms: timestamp,
                                event_index: event_idx as i64,
                            });
                        },
                        Err(e) => {
                            // Log but don't fail - might be a different event version
                            warn!("Failed to deserialize DIDClaimed event in tx {}: {}", 
                                  &tx_digest[..8], e);
                        }
                    }
                }
            }
            }
        }

        if self.log_config.should_log_events() && !events.is_empty() {
            info!("Processed {} DIDClaimed events from checkpoint {}", 
                  events.len(), checkpoint_seq);
        }

        Ok(events)
    }
}

#[async_trait::async_trait]
impl Handler for DIDClaimedEventHandler {
    type Store = Db;
    type Batch = Vec<StoredDIDClaimedEvent>;

    fn batch(&self, batch: &mut Vec<StoredDIDClaimedEvent>, values: std::vec::IntoIter<StoredDIDClaimedEvent>) {
        batch.extend(values);
    }

    async fn commit<'a>(
        &self,
        batch: &Vec<StoredDIDClaimedEvent>,
        conn: &mut <Db as Store>::Connection<'a>,
    ) -> Result<usize> {
        if batch.is_empty() {
            return Ok(0);
        }

        if self.log_config.should_log_detailed() {
            debug!("💾 Committing {} DIDClaimed events to database", batch.len());
        }

        // 1. Write to PostgreSQL (permanent storage)
        let inserted = diesel::insert_into(did_claimed_events)
            .values(batch)
            .on_conflict((transaction_digest, event_index))
            .do_nothing()
            .execute(conn)
            .await?;

        if self.log_config.should_log_events() {
            if inserted > 0 {
                info!("✅ Successfully inserted {} new DIDClaimed events to PostgreSQL", inserted);
            } else {
                debug!("No new events inserted (duplicates skipped)");
            }
        }

        // 2. Publish to Redis Pub/Sub (real-time notifications)
        // Note: If no subscribers, message is lost - this is acceptable
        if !batch.is_empty() {
            info!("🔍 Processing {} events for Redis publishing", batch.len());
            
            if let Ok(redis_url) = std::env::var("REDIS_URL") {
                if let Ok(redis_client) = redis::Client::open(redis_url) {
                    if let Ok(mut redis_con) = redis_client.get_multiplexed_async_connection().await {
                        for event in batch {
                            match serde_json::to_string(event) {
                                Ok(event_json) => {
                                    // PUBLISH to Redis Pub/Sub channel
                                    let _: Result<(), redis::RedisError> = redis_con.publish("did_claimed", event_json).await;
                                    
                                    if self.log_config.should_log_events() {
                                        info!("📤 Published event to Redis Pub/Sub channel 'did_claimed'");
                                    }
                                }
                                Err(e) => {
                                    warn!("⚠️  Failed to serialize event for Redis: {}", e);
                                }
                            }
                        }
                    } else {
                        warn!("⚠️  Failed to connect to Redis - events not published (subscribers will miss this)");
                    }
                } else {
                    warn!("⚠️  Invalid Redis URL - events not published");
                }
            } else {
                if self.log_config.should_log_detailed() {
                    debug!("No REDIS_URL configured - skipping Redis publish");
                }
            }
        }

        Ok(inserted)
    }
}
