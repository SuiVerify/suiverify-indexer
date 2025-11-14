use diesel::prelude::*;
use sui_indexer_alt_framework::FieldCount;
use crate::schema::{transaction_digests, did_claimed_events};

#[derive(Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = transaction_digests)]
pub struct StoredTransactionDigest {
    pub tx_digest: String,
    pub checkpoint_sequence_number: i64,
}

#[derive(Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = did_claimed_events)]
pub struct StoredDIDClaimedEvent {
    pub registry_id: String,
    pub user_address: String,
    pub did_type: i16,
    pub user_did_id: String,
    pub nft_id: String,
    pub checkpoint_sequence_number: i64,
    pub transaction_digest: String,
    pub timestamp_ms: i64,
    pub event_index: i64,
}
