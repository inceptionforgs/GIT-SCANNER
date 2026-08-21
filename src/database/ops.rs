use crate::database::mongo::MongoDB;
use crate::database::models::{MonitoredWalletDoc, ScanLogDoc};
use mongodb::bson::{doc, DateTime};
use mongodb::options::{FindOptions, UpdateOptions, ReplaceOptions};
use std::sync::Arc;
use chrono::Utc;
use tracing::{info, warn};

pub struct DatabaseOps {
    db: Arc<MongoDB>,
}

impl DatabaseOps {
    pub fn new(db: Arc<MongoDB>) -> Self {
        Self { db }
    }
    
    // Save monitored wallet (upsert - update if exists, insert if new)
    pub async fn save_monitored_wallet(
        &self,
        wallet: MonitoredWalletDoc,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.get_collection::<MonitoredWalletDoc>("monitored_wallets");
        
        let filter = doc! { "address": &wallet.address };
        let options = ReplaceOptions::builder().upsert(true).build();
        
        collection.replace_one(filter, &wallet, options).await?;
        
        info!("✅ Wallet saved: {}", wallet.address);
        Ok(())
    }
    
    // Bulk save monitored wallets
    pub async fn bulk_save_wallets(
        &self,
        wallets: Vec<MonitoredWalletDoc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if wallets.is_empty() {
            return Ok(());
        }
        
        let collection = self.db.get_collection::<MonitoredWalletDoc>("monitored_wallets");
        
        let mut bulk_ops = Vec::new();
        
        for wallet in wallets {
            let filter = doc! { "address": &wallet.address };
            let replace = mongodb::model::ReplaceOneModel::builder()
                .filter(filter)
                .replacement(wallet)
                .upsert(true)
                .build();
            
            bulk_ops.push(mongodb::model::WriteModel::ReplaceOne(replace));
        }
        
        collection.bulk_write(bulk_ops).await?;
        
        info!("✅ Bulk saved {} wallets", bulk_ops.len());
        Ok(())
    }
    
    // Get all monitoring wallets
    pub async fn get_monitoring_wallets(
        &self,
    ) -> Result<Vec<MonitoredWalletDoc>, Box<dyn std::error::Error>> {
        let collection = self.db.get_collection::<MonitoredWalletDoc>("monitored_wallets");
        
        let filter = doc! { "status": "monitoring" };
        
        let mut cursor = collection.find(filter).await?;
        
        let mut wallets = Vec::new();
        
        while cursor.advance().await? {
            if let Ok(wallet) = cursor.deserialize_current() {
                wallets.push(wallet);
            }
        }
        
        Ok(wallets)
    }
    
    // Update wallet after sweep
    pub async fn update_wallet_after_sweep(
        &self,
        address: &str,
        tx_hash: &str,
        amount: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.get_collection::<MonitoredWalletDoc>("monitored_wallets");
        
        let now = Utc::now();
        let bson_now = DateTime::from_millis(now.timestamp_millis());
        
        let filter = doc! { "address": address };
        
        let update = doc! {
            "$set": {
                "last_checked": bson_now,
                "last_balance": "0",
                "updated_at": bson_now,
            },
            "$push": {
                "tx_history": {
                    "tx_hash": tx_hash,
                    "amount": amount,
                    "to_address": "recipient",
                    "timestamp": bson_now,
                }
            },
            "$inc": {
                "total_swept": amount,
            }
        };
        
        collection.update_one(filter, update).await?;
        
        info!("✅ Wallet updated after sweep: {}", address);
        Ok(())
    }
    
    // Update last checked time
    pub async fn update_last_checked(
        &self,
        address: &str,
        balance: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.get_collection::<MonitoredWalletDoc>("monitored_wallets");
        
        let now = Utc::now();
        let bson_now = DateTime::from_millis(now.timestamp_millis());
        
        let filter = doc! { "address": address };
        
        let update = doc! {
            "$set": {
                "last_checked": bson_now,
                "last_balance": balance,
                "updated_at": bson_now,
            }
        };
        
        collection.update_one(filter, update).await?;
        
        Ok(())
    }
    
    // Save scan log
    pub async fn save_scan_log(
        &self,
        log: ScanLogDoc,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.get_collection::<ScanLogDoc>("scan_logs");
        
        collection.insert_one(log).await?;
        
        Ok(())
    }
    
    // Check if wallet exists
    pub async fn wallet_exists(
        &self,
        address: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let collection = self.db.get_collection::<MonitoredWalletDoc>("monitored_wallets");
        
        let filter = doc! { "address": address };
        
        let count = collection.count_documents(filter).await?;
        
        Ok(count > 0)
    }
}