use ethers::providers::{Http, Provider, Middleware};
use std::sync::Arc;

pub struct BalanceChecker {
    provider: Arc<Provider<Http>>,
}

impl BalanceChecker {
    pub fn new(rpc_url: &str) -> Result<Self, String> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| e.to_string())?;
        Ok(Self { provider: Arc::new(provider) })
    }
    
    pub async fn get_balance(&self, address: &str) -> Result<String, String> {
        let addr: ethers::types::Address = address.parse()
            .map_err(|e| e.to_string())?;
        let balance = self.provider.get_balance(addr, None).await
            .map_err(|e| e.to_string())?;
        Ok(ethers::utils::format_ether(balance))
    }
}