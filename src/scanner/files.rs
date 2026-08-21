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

// Fast string-based pre-check (no regex)
#[inline]
fn fast_skip_check(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    
    // Fast string checks (no regex overhead)
    lower.starts_with("node_modules/")
        || lower.starts_with(".git/")
        || lower.starts_with("dist/")
        || lower.starts_with("build/")
        || lower.starts_with("artifacts/")
        || lower.starts_with("cache/")
        || lower.ends_with(".min.js")
        || lower.ends_with(".map")
        || lower.ends_with(".lock")
        || lower.ends_with("package-lock.json")
        || lower.ends_with("yarn.lock")
}

// Fast string-based target check (no regex)
#[inline]
fn fast_target_check(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    
    // Fast string checks
    lower.ends_with(".env")
        || lower.ends_with(".env.local")
        || lower.ends_with(".env.production")
        || lower.ends_with(".git-credentials")
        || lower.ends_with(".secret")
        || lower.ends_with("id_rsa")
        || lower.ends_with("id_ed25519")
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower.ends_with("keystore.json")
        || lower.contains("hardhat.config")
        || lower.contains("truffle-config")
        || lower.contains("wallet.")
        || lower.contains("deploy.")
        || lower.contains("config.")
        || lower.contains("secrets.")
        || lower.contains("settings.")
}

// Check if file should be scanned (optimized)
pub fn should_scan_file(filename: &str) -> bool {
    // Fast string check first (no regex)
    if fast_skip_check(filename) {
        return false;
    }
    
    // Fast target check
    if fast_target_check(filename) {
        return true;
    }
    
    // Fallback to regex (only if string check inconclusive)
    if SKIP_FILE_PATTERNS.iter().any(|p| p.is_match(filename)) {
        return false;
    }
    
    TARGET_FILE_PATTERNS.iter().any(|p| p.is_match(filename))
}

// Get file priority (optimized with string checks)
pub fn get_file_priority(filename: &str) -> FilePriority {
    let lower = filename.to_lowercase();
    
    // Critical files — fast string check
    if lower.ends_with(".env") 
        || lower.ends_with(".env.local")
        || lower.ends_with(".env.production")
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