use ethers::providers::{Http, Provider, Middleware};
use std::sync::Arc;

pub struct BalanceChecker {
    provider: Arc<Provider<Http>>,
}

impl BalanceChecker {
    pub fn new(rpc_url: &str) -> Result<Self, String> {
        match Provider::<Http>::try_from(rpc_url) {
            Ok(provider) => Ok(Self { provider: Arc::new(provider) }),
            Err(e) => Err(format!("RPC error: {}", e)),
        }
    }
    
    pub async fn get_balance(&self, address: &str) -> Result<String, String> {
        match address.parse::<ethers::types::Address>() {
            Ok(addr) => {
                match self.provider.get_balance(addr, None).await {
                    Ok(balance) => Ok(ethers::utils::format_ether(balance)),
                    Err(e) => Err(format!("Balance error: {}", e)),
                }
            }
            Err(e) => Err(format!("Address error: {}", e)),
        }
    }
}