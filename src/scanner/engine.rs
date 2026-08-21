use crate::config::Config;
use crate::core::cache::CacheManager;
use crate::core::queue::EventQueue;
use crate::models::github::{CommitDetail, GitHubEvent};
use crate::patterns::matcher::PatternMatcher;
use crate::scanner::commits::CommitFetcher;
use crate::scanner::events::GitHubEventsPoller;
use crate::scanner::files::{should_scan_file, get_file_priority};
use crate::models::scan::{CryptoSecret, FilePriority};
use std::sync::Arc;
use tracing::{info, warn};

pub struct ScanEngine {
    config: Arc<Config>,
    cache: Arc<CacheManager>,
    matcher: Arc<PatternMatcher>,
    commit_queue: Arc<EventQueue<CommitDetail>>,
    secret_queue: Arc<EventQueue<(String, String, CryptoSecret)>>,
}

impl ScanEngine {
    pub fn new(
        config: Arc<Config>,
        cache: Arc<CacheManager>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config: config.clone(),
            cache,
            matcher: Arc::new(PatternMatcher::new()),
            commit_queue: EventQueue::new(50_000),
            secret_queue: EventQueue::new(10_000),
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
            
            while let Ok(event) = engine.commit_queue.receiver().recv_async().await {
                let commits = fetcher.fetch_commits_from_detail(&event).await;
                
                for commit in commits {
                    let _ = engine.secret_queue.sender().send_async(commit).await;
                }
            }
        });
        
        // Consumer 2: Scan commits (parallel)
        let engine = self.clone();
        let consumer2 = tokio::spawn(async move {
            while let Ok(commit) = engine.secret_queue.receiver().recv_async().await {
                let engine = engine.clone();
                tokio::spawn(async move {
                    engine.scan_commit(commit).await;
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
                    let _ = self.commit_queue.sender().send_async(event.clone()).await;
                    break; // One event per commit batch
                }
            }
        }
    }
    
    // Scan commit for secrets
    async fn scan_commit(&self, commit: CommitDetail) {
        let repo_name = commit.html_url.as_ref()
            .map(|url| url.replace("https://github.com/", "").split("/commit/").next().unwrap_or("unknown").to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
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
                        
                        // Send to secret queue for processing
                        let _ = self.secret_queue.sender().send_async((
                            repo_name.clone(),
                            commit.sha.clone(),
                            secret,
                        )).await;
                    }
                }
            }
        }
    }
}