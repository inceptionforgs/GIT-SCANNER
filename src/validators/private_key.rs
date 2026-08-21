use secp256k1::{Secp256k1, SecretKey};
use hex;
use tracing::warn;

pub struct PrivateKeyValidator;

impl PrivateKeyValidator {
    pub fn new() -> Self {
        Self
    }
    
    // Validate private key format
    pub fn validate_format(&self, private_key: &str) -> bool {
        // Remove 0x prefix if present
        let clean_key = private_key.trim_start_matches("0x");
        
        // Check length
        if clean_key.len() != 64 {
            return false;
        }
        
        // Check if all characters are hex
        if !clean_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
        
        true
    }
    
    // Validate if private key is cryptographically valid
    pub fn validate_cryptographic(&self, private_key: &str) -> bool {
        // Remove 0x prefix
        let clean_key = private_key.trim_start_matches("0x");
        
        // Check format first
        if !self.validate_format(clean_key) {
            return false;
        }
        
        // Decode hex
        let key_bytes = match hex::decode(clean_key) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        
        // Try to create secp256k1 secret key
        let secp = Secp256k1::new();
        match SecretKey::from_slice(&key_bytes) {
            Ok(_) => true,
            Err(e) => {
                warn!("Invalid secp256k1 key: {}", e);
                false
            }
        }
    }
    
    // Full validation (format + cryptographic)
    pub fn validate(&self, private_key: &str) -> (bool, f64) {
        let format_valid = self.validate_format(private_key);
        let crypto_valid = self.validate_cryptographic(private_key);
        
        // Confidence score
        let confidence = if format_valid && crypto_valid {
            1.0  // 100% valid
        } else if format_valid {
            0.5  // Format sahi but crypto invalid
        } else {
            0.0  // Invalid
        };
        
        (format_valid && crypto_valid, confidence)
    }
    
    // Check if key is within valid secp256k1 range
    pub fn is_in_valid_range(&self, private_key: &str) -> bool {
        let clean_key = private_key.trim_start_matches("0x");
        
        let key_bytes = match hex::decode(clean_key) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        
        // secp256k1 curve order (n)
        let curve_order = hex::decode("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141")
            .unwrap_or_default();
        
        // Key must be less than curve order
        key_bytes < curve_order
    }
}