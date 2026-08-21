use crate::config::Config;
use crate::core::cache::CacheManager;
use crate::models::github::GitHubEvent;
use crate::patterns::matcher::PatternMatcher;
use crate::scanner::commits::CommitFetcher;
use crate::scanner::events::GitHubEventsPoller;
use crate::scanner::files::{should_scan_file, get_file_priority};
use crate::models::scan::FilePriority;
use std::sync::Arc;
use tracing::{info, warn};

pub struct ScanEngine {
    config: Arc<Config>,
    cache: Arc<CacheManager>,
    matcher: Arc<PatternMatcher>,
}

impl ScanEngine {
    pub fn new(config: Arc<Config>, cache: Arc<CacheManager>) -> Arc<Self> {
        Arc::new(Self {
            config,
            cache,
            matcher: Arc::new(PatternMatcher::new()),
        })
    }
    
    pub async fn run(self: Arc<Self>) {
        info!("🚀 Scan engine started");
        
        let poller = GitHubEventsPoller::new(self.config.clone());
        
        poller.poll(move |event: GitHubEvent| {
            info!("📦 Event received: {} from {}", event.event_type, event.repo.name);
            
            // Simple processing for now
            if let Some(commits) = &event.payload.commits {
                for commit in commits {
                    info!("📝 Commit: {}", commit.sha);
                }
            }
        }).await;
    }
}