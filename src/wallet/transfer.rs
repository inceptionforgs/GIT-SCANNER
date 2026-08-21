use ethers::prelude::*;
use ethers::providers::{Http, Provider, Middleware};
use ethers::middleware::SignerMiddleware;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{info, warn};
use crate::models::wallet::TransferResult;

pub struct TransferExecutor {
    provider: Arc<Provider<Http>>,
    chain_id: u64,
    // Track nonce per address to avoid conflicts
    nonce_cache: Mutex<HashMap<String, u64>>,
    // Track pending transfers to avoid duplicate sends
    pending_transfers: Mutex<HashMap<String, bool>>,
}

impl TransferExecutor {
    pub fn new(rpc_url: &str, chain_id: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let provider = Arc::new(provider);
        
        Ok(Self {
            provider,
            chain_id,
            nonce_cache: Mutex::new(HashMap::new()),
            pending_transfers: Mutex::new(HashMap::new()),
        })
    }
    
    // Check if transfer is already pending for this address
    async fn is_pending(&self, address: &str) -> bool {
        let pending = self.pending_transfers.lock().await;
        pending.get(address).copied().unwrap_or(false)
    }
    
    // Mark address as pending
    async fn set_pending(&self, address: &str, status: bool) {
        let mut pending = self.pending_transfers.lock().await;
        pending.insert(address.to_string(), status);
    }
    
    // Get next nonce for address
    async fn get_next_nonce(&self, address: &str) -> Result<u64, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.nonce_cache.lock().await;
            if let Some(nonce) = cache.get(address) {
                return Ok(*nonce);
            }
        }
        
        // Fetch from network
        let addr: ethers::types::Address = address.parse()?;
        let nonce = self.provider.get_transaction_count(addr, None).await?;
        
        // Store in cache
        {
            let mut cache = self.nonce_cache.lock().await;
            cache.insert(address.to_string(), nonce.as_u64());
        }
        
        Ok(nonce.as_u64())
    }
    
    // Increment nonce after successful send
    async fn increment_nonce(&self, address: &str) {
        let mut cache = self.nonce_cache.lock().await;
        if let Some(nonce) = cache.get_mut(address) {
            *nonce += 1;
        }
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
        
        // Get address
        let from_address = format!("{:?}", wallet.address());
        
        // Check if transfer already pending
        if self.is_pending(&from_address).await {
            warn!("Transfer already pending for {}", from_address);
            return Ok(TransferResult {
                from_address: from_address.clone(),
                to_address: to_address.to_string(),
                amount: amount.to_string(),
                tx_hash: String::new(),
                success: false,
                error: Some("Transfer already pending".to_string()),
            });
        }
        
        // Mark as pending
        self.set_pending(&from_address, true).await;
        
        // Create signer middleware
        let client = SignerMiddleware::new(
            self.provider.clone(),
            wallet.clone(),
        );
        
        // Parse addresses and amount
        let to: ethers::types::Address = to_address.parse()?;
        let amount_wei = ethers::utils::parse_ether(amount)?;
        
        // Get nonce
        let nonce = self.get_next_nonce(&from_address).await?;
        
        // Build transaction
        let tx = TransactionRequest::new()
            .from(wallet.address())
            .to(to)
            .value(amount_wei)
            .gas(gas_limit)
            .gas_price(ethers::utils::parse_units(gas_price_gwei, "gwei")?)
            .nonce(nonce);
        
        info!("📤 Sending transaction from {} to {} (nonce: {})", from_address, to_address, nonce);
        
        // Send transaction
        let result = match client.send_transaction(tx, None).await {
            Ok(pending_tx) => {
                let tx_hash = format!("{:?}", pending_tx.tx_hash());
                
                info!("✅ Transaction sent: {}", tx_hash);
                
                // Increment nonce for next transaction
                self.increment_nonce(&from_address).await;
                
                TransferResult {
                    from_address: from_address.clone(),
                    to_address: to_address.to_string(),
                    amount: amount.to_string(),
                    tx_hash,
                    success: true,
                    error: None,
                }
            }
            Err(e) => {
                warn!("❌ Transaction failed: {}", e);
                
                TransferResult {
                    from_address: from_address.clone(),
                    to_address: to_address.to_string(),
                    amount: amount.to_string(),
                    tx_hash: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                }
            }
        };
        
        // Clear pending status
        self.set_pending(&from_address, false).await;
        
        Ok(result)
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