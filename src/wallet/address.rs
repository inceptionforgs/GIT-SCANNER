use ethers::prelude::*;
use coins_bip39::English;
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
    
    pub fn derive_from_seed_phrase(&self, seed_phrase: &str) -> Option<String> {
        // Use coins_bip39 directly
        let mnemonic = match coins_bip39::Mnemonic::<English>::new_from_phrase(seed_phrase) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to parse mnemonic: {}", e);
                return None;
            }
        };
        
        // Derive first address (simplified)
        let private_key = mnemonic.derive_key("m/44'/60'/0'/0/0", None)
            .ok()?;
        
        let hex_key = hex::encode(private_key);
        self.derive_address(&hex_key)
    }
    
    pub fn derive_multiple_from_seed(&self, seed_phrase: &str, count: u32) -> Vec<(String, String)> {
        let mut results = Vec::new();
        
        let mnemonic = match coins_bip39::Mnemonic::<English>::new_from_phrase(seed_phrase) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to parse mnemonic: {}", e);
                return results;
            }
        };
        
        for index in 0..count {
            let path = format!("m/44'/60'/0'/0/{}", index);
            
            if let Ok(private_key) = mnemonic.derive_key(&path, None) {
                let hex_key = hex::encode(private_key);
                
                if let Some(address) = self.derive_address(&hex_key) {
                    results.push((address, hex_key));
                }
            }
        }
        
        results
    }
}