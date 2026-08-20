use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub actor: Actor,
    pub repo: Repo,
    pub payload: Payload,
    pub created_at: Option<String>,
    pub public: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Actor {
    pub id: u64,
    pub login: String,
    pub display_login: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Repo {
    pub id: u64,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Payload {
    pub push_id: Option<u64>,
    pub size: Option<u32>,
    pub distinct_size: Option<u32>,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub head: Option<String>,
    pub before: Option<String>,
    pub commits: Option<Vec<Commit>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Commit {
    pub sha: String,
    pub author: Option<CommitAuthor>,
    pub message: Option<String>,
    pub distinct: Option<bool>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommitAuthor {
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommitDetail {
    pub sha: String,
    pub commit: Option<CommitInfo>,
    pub files: Option<Vec<FileChange>>,
    pub html_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommitInfo {
    pub message: Option<String>,
    pub author: Option<CommitAuthorInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommitAuthorInfo {
    pub name: Option<String>,
    pub email: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileChange {
    pub sha: Option<String>,
    pub filename: String,
    pub status: Option<String>,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
    pub changes: Option<u32>,
    pub patch: Option<String>,
    pub raw_url: Option<String>,
}