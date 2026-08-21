use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_scanner::wallet::address::AddressDeriver;
use git_scanner::validators::private_key::PrivateKeyValidator;
use git_scanner::validators::seed_phrase::SeedPhraseValidator;

fn benchmark_address_derivation(c: &mut Criterion) {
    let deriver = AddressDeriver::new();
    
    let test_keys = vec![
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        "0x1111111111111111111111111111111111111111111111111111111111111111",
    ];
    
    c.bench_function("derive_address", |b| {
        b.iter(|| {
            for key in &test_keys {
                let address = deriver.derive_address(black_box(key));
                black_box(address);
            }
        })
    });
}

fn benchmark_private_key_validation(c: &mut Criterion) {
    let validator = PrivateKeyValidator::new();
    
    let valid_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    let invalid_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeg";
    
    c.bench_function("validate_private_key", |b| {
        b.iter(|| {
            let valid = validator.validate(black_box(valid_key));
            let invalid = validator.validate(black_box(invalid_key));
            black_box((valid, invalid));
        })
    });
}

fn benchmark_seed_phrase_validation(c: &mut Criterion) {
    let validator = SeedPhraseValidator::new();
    
    let valid_seed = "abandon ability able about above absent absorb abstract absurd abuse access accident";
    let invalid_seed = "hello world this is not a valid seed phrase";
    
    c.bench_function("validate_seed_phrase", |b| {
        b.iter(|| {
            let valid = validator.validate(black_box(valid_seed));
            let invalid = validator.validate(black_box(invalid_seed));
            black_box((valid, invalid));
        })
    });
}

criterion_group!(
    benches,
    benchmark_address_derivation,
    benchmark_private_key_validation,
    benchmark_seed_phrase_validation
);
criterion_main!(benches);