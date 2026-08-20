use once_cell::sync::Lazy;
use regex::Regex;

// Pre-compiled regex patterns for seed phrases (12/24 words)
pub static SEED_PHRASE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Pattern 1: mnemonic = "word1 word2 ... word12"
        Regex::new(r"(?i)(mnemonic|seed[_\-]?phrase|recovery[_\-]?phrase|words|phrase)\s*[=:]\s*['\"]((?:[a-z]{3,}\s){11}[a-z]{3,})['\"]").unwrap(),
        
        // Pattern 2: mnemonic = "word1 word2 ... word24"
        Regex::new(r"(?i)(mnemonic|seed[_\-]?phrase|recovery[_\-]?phrase|words|phrase)\s*[=:]\s*['\"]((?:[a-z]{3,}\s){23}[a-z]{3,})['\"]").unwrap(),
        
        // Pattern 3: Generic quoted 12 words
        Regex::new(r"['\"]((?:[a-z]{3,}\s){11}[a-z]{3,})['\"]").unwrap(),
        
        // Pattern 4: Generic quoted 24 words
        Regex::new(r"['\"]((?:[a-z]{3,}\s){23}[a-z]{3,})['\"]").unwrap(),
    ]
});

// Extract clean seed phrase from matched text
pub fn extract_seed_phrase(text: &str) -> Option<String> {
    // Split by whitespace and filter alphabetic words
    let words: Vec<&str> = text
        .split_whitespace()
        .filter(|w| {
            !w.is_empty() 
            && w.chars().all(|c| c.is_alphabetic())
            && w.len() >= 3
        })
        .collect();
    
    // Only accept 12 or 24 words
    if words.len() == 12 || words.len() == 24 {
        Some(words.join(" "))
    } else {
        None
    }
}

// Check if word count is valid
pub fn is_valid_word_count(phrase: &str) -> bool {
    let count = phrase.split_whitespace().count();
    count == 12 || count == 24
}