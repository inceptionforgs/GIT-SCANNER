use crate::config::Config;
use crate::models::github::{CommitDetail, GitHubEvent};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use futures::stream::{self, StreamExt};
use tracing::{info, warn};

pub struct CommitFetcher {
    client: Client,
    config: Arc<Config>,
}

// Extended commit detail with repo info
#[derive(Debug, Clone)]
pub struct CommitWithRepo {
    pub commit: CommitDetail,
    pub repo_name: String,
}

impl CommitFetcher {
    pub fn new(config: Arc<Config>) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(20)
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(15))
            .user_agent("git-scanner/1.0")
            .build()
            .unwrap();
        
        Self { client, config }
    }
    
    // Fetch commit details with repo info
    pub async fn fetch_commits(&self, event: &GitHubEvent) -> Vec<CommitWithRepo> {
        let repo_name = event.repo.name.clone();
        
        let commits = match &event.payload.commits {
            Some(commits) => commits.clone(),
            None => return vec![],
        };
        
        // Parallel fetch with buffer_unordered
        let results = stream::iter(commits)
            .map(|commit| {
                let client = self.client.clone();
                let config = self.config.clone();
                let repo_name = repo_name.clone();
                let sha = commit.sha.clone();
                
                async move {
                    fetch_single_commit(client, config, &repo_name, &sha).await
                }
            })
            .buffer_unordered(50)
            .collect::<Vec<_>>()
            .await;
        
        // Filter successful fetches and add repo name
        let commit_details: Vec<CommitWithRepo> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .flatten()
            .map(|commit| CommitWithRepo {
                commit,
                repo_name: repo_name.clone(),
            })
            .collect();
        
        info!("📦 Fetched {} commit details", commit_details.len());
        commit_details
    }
}

async fn fetch_single_commit(
    client: Client,
    config: Arc<Config>,
    repo_name: &str,
    sha: &str,
) -> Result<Option<CommitDetail>, reqwest::Error> {
    let url = format!(
        "{}/repos/{}/commits/{}",
        config.github_api_url, repo_name, sha
    );
    
    let request = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    
    let request = if let Some(token) = &config.github_token {
        request.header("Authorization", format!("Bearer {}", token))
    } else {
        request
    };
    
    let response = request.send().await?;
    
    if response.status().is_success() {
        let commit = response.json::<CommitDetail>().await?;
        Ok(Some(commit))
    } else {
        warn!("Failed to fetch commit {}: {}", sha, response.status());
        Ok(None)
    }
}