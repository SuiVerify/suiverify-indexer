use std::sync::Arc;
use anyhow::Result;
use sui_indexer_alt_framework::{
    pipeline::Processor,
};
use sui_types::full_checkpoint_content::Checkpoint;

use crate::models::StoredTransactionDigest;
use crate::schema::transaction_digests::dsl::*;
use diesel_async::RunQueryDsl;
use sui_indexer_alt_framework::{
    postgres::Db,
    pipeline::sequential::Handler,
    store::Store,
};

pub struct TransactionDigestHandler;

#[async_trait::async_trait]
impl Processor for TransactionDigestHandler {
    const NAME: &'static str = "transaction_digest_handler";

    type Value = StoredTransactionDigest;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<StoredTransactionDigest>> {
        let checkpoint_seq = checkpoint.summary.sequence_number as i64;

        let digests = checkpoint.transactions.iter().map(|tx| {
            StoredTransactionDigest {
                tx_digest: tx.transaction.digest().to_string(),
                checkpoint_sequence_number: checkpoint_seq,
            }
        }).collect();

        Ok(digests)
    }
}

#[async_trait::async_trait]
impl Handler for TransactionDigestHandler {
    type Store = Db;
    type Batch = Vec<StoredTransactionDigest>;

    fn batch(&self, batch: &mut Vec<StoredTransactionDigest>, values: std::vec::IntoIter<StoredTransactionDigest>) {
        batch.extend(values);
    }

    async fn commit<'a>(
        &self,
        batch: &Vec<StoredTransactionDigest>,
        conn: &mut <Db as Store>::Connection<'a>,
    ) -> Result<usize> {
        let inserted = diesel::insert_into(transaction_digests)
            .values(batch)
            .on_conflict(tx_digest)
            .do_nothing()
            .execute(conn)
            .await?;

        Ok(inserted)
    }
}
