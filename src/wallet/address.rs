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
    
    // Seed phrase ke liye simple approach — pehla address
    pub fn derive_from_seed_phrase(&self, seed_phrase: &str) -> Option<String> {
        // Use ethers MnemonicBuilder directly
        let wallet = match MnemonicBuilder::<English>::default()
            .phrase(seed_phrase)
            .build()
        {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to build wallet from seed phrase: {}", e);
                return None;
            }
        };
        
        let address = wallet.address();
        info!("✅ Address derived from seed: {:?}", address);
        
        Some(format!("{:?}", address))
    }
    
    pub fn derive_multiple_from_seed(&self, seed_phrase: &str, count: u32) -> Vec<(String, String)> {
        let mut results = Vec::new();
        
        for index in 0..count {
            let path = format!("m/44'/60'/0'/0/{}", index);
            
            match MnemonicBuilder::<English>::default()
                .phrase(seed_phrase)
                .derivation_path(&path)
                .build()
            {
                Ok(wallet) => {
                    let address = format!("{:?}", wallet.address());
                    let private_key = format!("{:x}", wallet.signer());
                    results.push((address, private_key));
                }
                Err(e) => {
                    warn!("Failed to derive address at index {}: {}", index, e);
                }
            }
        }
        
        results
    }
}