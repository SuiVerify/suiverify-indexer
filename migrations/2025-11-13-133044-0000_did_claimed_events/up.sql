CREATE TABLE IF NOT EXISTS did_claimed_events (
    id BIGSERIAL PRIMARY KEY,
    registry_id TEXT NOT NULL,
    user_address TEXT NOT NULL,
    did_type SMALLINT NOT NULL,
    user_did_id TEXT NOT NULL,
    nft_id TEXT NOT NULL,
    checkpoint_sequence_number BIGINT NOT NULL,
    transaction_digest TEXT NOT NULL,
    timestamp_ms BIGINT NOT NULL,
    event_index BIGINT NOT NULL,
    
    -- Indexes for efficient queries
    UNIQUE(transaction_digest, event_index)
);

CREATE INDEX idx_did_claimed_user ON did_claimed_events(user_address);
CREATE INDEX idx_did_claimed_nft ON did_claimed_events(nft_id);
CREATE INDEX idx_did_claimed_checkpoint ON did_claimed_events(checkpoint_sequence_number);
