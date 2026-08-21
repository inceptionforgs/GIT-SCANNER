use ethers::providers::{Http, Provider, Middleware};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

pub struct BalanceChecker {
    provider: Arc<Provider<Http>>,
}

impl BalanceChecker {
    pub fn new(rpc_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let provider = Arc::new(provider);
        
        Ok(Self { provider })
    }
    
    // Check balance for single address
    pub async fn get_balance(&self, address: &str) -> Result<String, Box<dyn std::error::Error>> {
        let address: ethers::types::Address = address.parse()?;
        
        let balance = self.provider.get_balance(address, None).await?;
        
        // Convert wei to eth
        let eth_balance = ethers::utils::format_ether(balance);
        
        Ok(eth_balance)
    }
    
    // Batch balance check for multiple addresses
    pub async fn get_balances_batch(
        &self,
        addresses: &[String],
    ) -> Vec<(String, Result<String, String>)> {
        use futures::stream::{self, StreamExt};
        
        let results = stream::iter(addresses)
            .map(|address| {
                let provider = self.provider.clone();
                let address = address.clone();
                
                async move {
                    let addr: ethers::types::Address = match address.parse() {
                        Ok(a) => a,
                        Err(e) => return (address, Err(e.to_string())),
                    };
                    
                    match provider.get_balance(addr, None).await {
                        Ok(balance) => {
                            let eth = ethers::utils::format_ether(balance);
                            (address, Ok(eth))
                        }
                        Err(e) => (address, Err(e.to_string())),
                    }
                }
            })
            .buffer_unordered(50) // 50 parallel balance checks
            .collect::<Vec<_>>()
            .await;
        
        results
    }
}