use moka::sync::Cache;
use std::time::Duration;
use std::sync::Arc;

pub struct CacheManager {
    scanned_commits: Cache<String, bool>,
    checked_addresses: Cache<String, String>,
}

impl CacheManager {
    pub fn new() -> Arc<Self> {
        let scanned_commits = Cache::builder()
            .max_capacity(100_000)
            .time_to_live(Duration::from_secs(3600))
            .build();
        
        let checked_addresses = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(30))
            .build();
        
        Arc::new(Self {
            scanned_commits,
            checked_addresses,
        })
    }
    
    pub fn should_scan_commit(&self, sha: &str) -> bool {
        if self.scanned_commits.contains_key(sha) {
            return false;
        }
        self.scanned_commits.insert(sha.to_string(), true);
        true
    }
    
    pub fn get_cached_balance(&self, address: &str) -> Option<String> {
        self.checked_addresses.get(address)
    }
    
    pub fn set_cached_balance(&self, address: &str, balance: String) {
        self.checked_addresses.insert(address.to_string(), balance);
    }
}