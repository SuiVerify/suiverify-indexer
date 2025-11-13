mod models;
mod handlers;

use handlers::TransactionDigestHandler;

pub mod schema;

use anyhow::Result;
use clap::Parser;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use sui_indexer_alt_framework::{
    cluster::{Args, IndexerCluster},
    pipeline::sequential::SequentialConfig,
};
use tokio;
use url::Url;

// Embed database migrations into the binary
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env data
    dotenvy::dotenv().ok();

    // Database URL
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in the environment")
        .parse::<Url>()
        .expect("Invalid database URL");
    
    // Parse command-line arguments
    let args = Args::parse();
    
    // Build and configure the indexer cluster
    let mut cluster = IndexerCluster::builder()
        .with_args(args)
        .with_database_url(database_url)
        .with_migrations(&MIGRATIONS)
        .build()
        .await?;
    
    // Register our custom sequential pipeline
    cluster.sequential_pipeline(
        TransactionDigestHandler,
        SequentialConfig::default(),
    ).await?;
    
    // Start the indexer
    let handle = cluster.run().await?;
    handle.await?;
    
    Ok(())
}
