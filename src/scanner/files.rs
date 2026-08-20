use once_cell::sync::Lazy;
use regex::Regex;
use crate::models::scan::FilePriority;

// Target files — sirf inhe scan karo (speed ke liye)
pub static TARGET_FILE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Critical files
        Regex::new(r"(?i)\.env$").unwrap(),
        Regex::new(r"(?i)\.env\..+$").unwrap(),
        Regex::new(r"(?i)\.git-credentials$").unwrap(),
        Regex::new(r"(?i)\.secret$").unwrap(),
        Regex::new(r"(?i)id_rsa$").unwrap(),
        Regex::new(r"(?i)id_ed25519$").unwrap(),
        Regex::new(r"(?i)\.pem$").unwrap(),
        Regex::new(r"(?i)\.key$").unwrap(),
        Regex::new(r"(?i)keystore\.json$").unwrap(),
        
        // High priority files
        Regex::new(r"(?i)hardhat\.config\.(js|ts)$").unwrap(),
        Regex::new(r"(?i)truffle-config\.(js|ts)$").unwrap(),
        Regex::new(r"(?i)wallet\.(js|ts|json)$").unwrap(),
        Regex::new(r"(?i)deploy\.(js|ts)$").unwrap(),
        Regex::new(r"(?i)config\.(js|ts|json|py|yaml|yml)$").unwrap(),
        Regex::new(r"(?i)secrets?\.(json|js|ts)$").unwrap(),
        Regex::new(r"(?i)settings\.(py|js|ts|json)$").unwrap(),
    ]
});

// Skip patterns — in files ko bilkul ignore karo
pub static SKIP_FILE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"^node_modules/").unwrap(),
        Regex::new(r"^\.git/").unwrap(),
        Regex::new(r"^dist/").unwrap(),
        Regex::new(r"^build/").unwrap(),
        Regex::new(r"^artifacts/").unwrap(),
        Regex::new(r"^cache/").unwrap(),
        Regex::new(r"\.min\.js$").unwrap(),
        Regex::new(r"\.map$").unwrap(),
        Regex::new(r"\.lock$").unwrap(),
        Regex::new(r"package-lock\.json$").unwrap(),
        Regex::new(r"yarn\.lock$").unwrap(),
    ]
});

// Check if file should be scanned
pub fn should_scan_file(filename: &str) -> bool {
    // Skip check first (fast rejection)
    if SKIP_FILE_PATTERNS.iter().any(|p| p.is_match(filename)) {
        return false;
    }
    
    // Target file check
    TARGET_FILE_PATTERNS.iter().any(|p| p.is_match(filename))
}

// Get file priority
pub fn get_file_priority(filename: &str) -> FilePriority {
    let lower = filename.to_lowercase();
    
    // Critical files
    if lower.ends_with(".env") 
        || lower.contains(".git-credentials")
        || lower.ends_with("id_rsa")
        || lower.ends_with("id_ed25519")
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower.contains("keystore.json")
        || lower.ends_with(".secret") {
        return FilePriority::Critical;
    }
    
    // High priority
    if lower.contains("hardhat")
        || lower.contains("truffle")
        || lower.contains("wallet")
        || lower.contains("deploy")
        || lower.contains("config")
        || lower.contains("secrets")
        || lower.contains("settings") {
        return FilePriority::High;
    }
    
    FilePriority::Medium
}