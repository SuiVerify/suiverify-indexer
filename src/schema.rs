// @generated automatically by Diesel CLI.

diesel::table! {
    did_claimed_events (id) {
        id -> Int8,
        registry_id -> Text,
        user_address -> Text,
        did_type -> Int2,
        user_did_id -> Text,
        nft_id -> Text,
        checkpoint_sequence_number -> Int8,
        transaction_digest -> Text,
        timestamp_ms -> Int8,
        event_index -> Int8,
    }
}

diesel::table! {
    transaction_digests (tx_digest) {
        tx_digest -> Text,
        checkpoint_sequence_number -> Int8,
    }
}

diesel::table! {
    watermarks (pipeline) {
        pipeline -> Text,
        epoch_hi_inclusive -> Int8,
        checkpoint_hi_inclusive -> Int8,
        tx_hi -> Int8,
        timestamp_ms_hi_inclusive -> Int8,
        reader_lo -> Int8,
        pruner_timestamp -> Timestamp,
        pruner_hi -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(did_claimed_events, transaction_digests, watermarks,);
