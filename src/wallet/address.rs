use ethers::prelude::*;
use tracing::{info, warn};

pub struct AddressDeriver;

impl AddressDeriver {
    pub fn new() -> Self {
        Self
    }
    
    pub fn derive_address(&self, private_key: &str) -> Option<String> {
        let clean_key = private_key.trim_start_matches("0x");
        
        if clean_key.len() != 64 {
            warn!("Invalid private key length: {}", clean_key.len());
            return None;
        }
        
        if !clean_key.chars().all(|c| c.is_ascii_hexdigit()) {
            warn!("Invalid private key: non-hex characters");
            return None;
        }
        
        let wallet: LocalWallet = match clean_key.parse() {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to parse private key: {}", e);
                return None;
            }
        };
        
        let address = wallet.address();
        info!("✅ Address derived: {:?}", address);
        
        Some(format!("{:?}", address))
    }
    
    // Seed phrase ke liye — temporarily disabled
    pub fn derive_from_seed_phrase(&self, _seed_phrase: &str) -> Option<String> {
        warn!("Seed phrase derivation temporarily disabled");
        None
    }
    
    pub fn derive_multiple_from_seed(&self, _seed_phrase: &str, _count: u32) -> Vec<(String, String)> {
        warn!("Seed phrase derivation temporarily disabled");
        vec![]
    }
}