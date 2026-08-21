use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_scanner::patterns::matcher::PatternMatcher;

fn benchmark_private_key_scan(c: &mut Criterion) {
    let matcher = PatternMatcher::new();
    
    let test_content = r#"
        PRIVATE_KEY=0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
        API_KEY=sk_test_1234567890abcdef
        password=mysecretpassword123
        
        const config = {
            privateKeys: ["0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"],
            apiKey: "random_api_key_here"
        }
        
        MNEMONIC="abandon ability able about above absent absorb abstract absurd abuse access accident"
        
        # Some random text
        Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
    "#;
    
    c.bench_function("scan_private_keys", |b| {
        b.iter(|| {
            let secrets = matcher.scan_content(black_box(test_content));
            black_box(secrets)
        })
    });
}

fn benchmark_seed_phrase_scan(c: &mut Criterion) {
    let matcher = PatternMatcher::new();
    
    let test_content = r#"
        MNEMONIC="abandon ability able about above absent absorb abstract absurd abuse access accident"
        SEED_PHRASE="legal winner thank year wave sausage worth useful legal winner thank yellow"
        
        const wallet = {
            mnemonic: "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
            privateKey: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        }
    "#;
    
    c.bench_function("scan_seed_phrases", |b| {
        b.iter(|| {
            let secrets = matcher.scan_content(black_box(test_content));
            black_box(secrets)
        })
    });
}

fn benchmark_large_content_scan(c: &mut Criterion) {
    let matcher = PatternMatcher::new();
    
    // Generate large content (1MB)
    let mut large_content = String::with_capacity(1_000_000);
    for i in 0..10_000 {
        large_content.push_str(&format!(
            "Line {}: PRIVATE_KEY=0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\n",
            i
        ));
        large_content.push_str(&format!(
            "Line {}: random text here\n",
            i
        ));
    }
    
    c.bench_function("scan_large_content_1mb", |b| {
        b.iter(|| {
            let secrets = matcher.scan_content(black_box(&large_content));
            black_box(secrets)
        })
    });
}

criterion_group!(
    benches,
    benchmark_private_key_scan,
    benchmark_seed_phrase_scan,
    benchmark_large_content_scan
);
criterion_main!(benches);