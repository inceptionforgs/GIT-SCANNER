use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum SecretType {
    PrivateKey,
    SeedPhrase,
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretType::PrivateKey => write!(f, "PrivateKey"),
            SecretType::SeedPhrase => write!(f, "SeedPhrase"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CryptoSecret {
    pub secret_type: SecretType,
    pub value: String,
    pub raw_match: String,
    pub line_number: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanResult {
    pub id: String,
    pub repository: String,
    pub commit_sha: String,
    pub file_path: String,
    pub secret_type: String,
    pub matched_value: String,
    pub confidence_score: f64,
    pub line_number: Option<u32>,
    pub detected_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilePriority {
    Critical = 1,
    High = 2,
    Medium = 3,
    Low = 4,
    Skip = 5,
}