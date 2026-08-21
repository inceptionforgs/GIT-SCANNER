use ethers::prelude::*;
use tracing::{info, warn};

pub struct AddressDeriver;

impl AddressDeriver {
    pub fn new() -> Self {
        Self
    }
    
    // Derive wallet address from private key
    pub fn derive_address(&self, private_key: &str) -> Option<String> {
        // Clean private key (remove 0x prefix)
        let clean_key = private_key.trim_start_matches("0x");
        
        // Validate length
        if clean_key.len() != 64 {
            warn!("Invalid private key length: {} (expected 64)", clean_key.len());
            return None;
        }
        
        // Validate hex characters
        if !clean_key.chars().all(|c| c.is_ascii_hexdigit()) {
            warn!("Invalid private key: non-hex characters found");
            return None;
        }
        
        // Parse private key to wallet
        let wallet: LocalWallet = match clean_key.parse() {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to parse private key: {}", e);
                return None;
            }
        };
        
        // Return address as checksummed string
        let address = wallet.address();
        info!("✅ Address derived: {:?}", address);
        
        Some(format!("{:?}", address))
    }
    
    // Derive address from seed phrase (first account)
    pub fn derive_from_seed_phrase(&self, seed_phrase: &str) -> Option<String> {
        // Parse mnemonic
        let mnemonic = match Mnemonic::from_phrase(seed_phrase, None) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to parse mnemonic: {}", e);
                return None;
            }
        };
        
        // Build wallet from mnemonic (default derivation path)
        let wallet = match MnemonicBuilder::<English>::default()
            .mnemonic(mnemonic)
            .build()
        {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to build wallet from mnemonic: {}", e);
                return None;
            }
        };
        
        let address = wallet.address();
        info!("✅ Address derived from seed: {:?}", address);
        
        Some(format!("{:?}", address))
    }
    
    // Derive multiple addresses from seed phrase
    pub fn derive_multiple_from_seed(&self, seed_phrase: &str, count: u32) -> Vec<(String, String)> {
        let mut results = Vec::new();
        
        let mnemonic = match Mnemonic::from_phrase(seed_phrase, None) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to parse mnemonic: {}", e);
                return results;
            }
        };
        
        for index in 0..count {
            let derivation_path = format!("m/44'/60'/0'/0/{}", index);
            
            match MnemonicBuilder::<English>::default()
                .mnemonic(mnemonic.clone())
                .derivation_path(&derivation_path)
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
    
    // Check if address is valid Ethereum address
    pub fn is_valid_address(&self, address: &str) -> bool {
        address.parse::<ethers::types::Address>().is_ok()
    }
    
    // Convert private key to bytes
    pub fn private_key_to_bytes(&self, private_key: &str) -> Option<[u8; 32]> {
        let clean_key = private_key.trim_start_matches("0x");
        
        if clean_key.len() != 64 {
            return None;
        }
        
        let mut bytes = [0u8; 32];
        
        match hex::decode_to_slice(clean_key, &mut bytes) {
            Ok(_) => Some(bytes),
            Err(e) => {
                warn!("Failed to decode private key: {}", e);
                None
            }
        }
    }
}