mod models;
mod handlers;
mod events;
mod event_handlers;
mod config;

use handlers::TransactionDigestHandler;
use event_handlers::DIDClaimedEventHandler;
use config::LogConfig;

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
use tracing::info;

// Embed database migrations into the binary
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env data
    dotenvy::dotenv().ok();

    // Initialize logging configuration
    let log_config = LogConfig::from_env();
    
    // The framework already sets up tracing, so we just use it directly
    // Our logging configuration will control what gets logged

    if log_config.should_log_detailed() {
        info!("Starting SuiVerify Indexer with detailed logging enabled");
        info!("Log configuration: {:?}", log_config);
    }

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
    // cluster.sequential_pipeline(
    //     TransactionDigestHandler,
    //     SequentialConfig::default(),
    // ).await?;
    
    // if log_config.should_log_detailed() {
    //     info!("Registered TransactionDigestHandler pipeline");
    // }
    
    // Register DIDClaimed event pipeline
    cluster.sequential_pipeline(
        DIDClaimedEventHandler::new(log_config.clone()),
        SequentialConfig::default(),
    ).await?;
    
    if log_config.should_log_detailed() {
        info!("Registered DIDClaimedEventHandler pipeline");
        info!("Monitoring events for package: 0x6ec40d30e636afb906e621748ee60a9b72bc59a39325adda43deadd28dc89e09");
    }
    
    // Start the indexer
    let handle = cluster.run().await?;
    handle.await?;
    
    Ok(())
}
