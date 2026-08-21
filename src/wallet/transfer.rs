use ethers::prelude::*;
use ethers::providers::{Http, Provider, Middleware};
use ethers::middleware::SignerMiddleware;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};
use crate::models::wallet::TransferResult;

pub struct TransferExecutor {
    provider: Arc<Provider<Http>>,
    chain_id: u64,
}

impl TransferExecutor {
    pub fn new(rpc_url: &str, chain_id: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let provider = Arc::new(provider);
        
        Ok(Self { provider, chain_id })
    }
    
    // Transfer native token (ETH/BNB/MATIC)
    pub async fn transfer_native(
        &self,
        private_key: &str,
        to_address: &str,
        amount: &str,
        gas_limit: u64,
        gas_price_gwei: u64,
    ) -> Result<TransferResult, Box<dyn std::error::Error>> {
        // Parse private key
        let wallet: LocalWallet = private_key.trim_start_matches("0x").parse()?;
        let wallet = wallet.with_chain_id(self.chain_id);
        
        // Create signer middleware
        let client = SignerMiddleware::new(
            self.provider.clone(),
            wallet.clone(),
        );
        
        // Parse addresses and amount
        let from_address = format!("{:?}", wallet.address());
        let to: ethers::types::Address = to_address.parse()?;
        let amount_wei = ethers::utils::parse_ether(amount)?;
        
        // Build transaction
        let tx = TransactionRequest::new()
            .from(wallet.address())
            .to(to)
            .value(amount_wei)
            .gas(gas_limit)
            .gas_price(ethers::utils::parse_units(gas_price_gwei, "gwei")?);
        
        info!("📤 Sending transaction from {} to {}", from_address, to_address);
        
        // Send transaction
        match client.send_transaction(tx, None).await {
            Ok(pending_tx) => {
                let tx_hash = format!("{:?}", pending_tx.tx_hash());
                
                info!("✅ Transaction sent: {}", tx_hash);
                
                Ok(TransferResult {
                    from_address,
                    to_address: to_address.to_string(),
                    amount: amount.to_string(),
                    tx_hash,
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                warn!("❌ Transaction failed: {}", e);
                
                Ok(TransferResult {
                    from_address,
                    to_address: to_address.to_string(),
                    amount: amount.to_string(),
                    tx_hash: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                })
            }
        }
    }
    
    // Calculate max transferable amount (balance - gas)
    pub fn calculate_max_amount(balance: &str, gas_price_gwei: u64, gas_limit: u64) -> String {
        let balance_eth: f64 = balance.parse().unwrap_or(0.0);
        let gas_cost = (gas_price_gwei as f64 * gas_limit as f64) / 1_000_000_000.0;
        
        let max_amount = balance_eth - gas_cost;
        
        if max_amount > 0.0 {
            format!("{:.6}", max_amount)
        } else {
            "0".to_string()
        }
    }
}