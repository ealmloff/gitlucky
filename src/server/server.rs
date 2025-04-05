use axum::{routing::post, Json, Router};

#[cfg(not(feature = "server"))]
use dioxus::prelude::{DioxusRouterExt, ServeConfig};
use octocrab::models::events::payload::PullRequestEventPayload;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

#[cfg(not(feature = "server"))]
use crate::App;

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub diff_url: String,
    pub title: String,
    pub additions: usize,
    pub deletions: usize,
    pub changed_files: usize,
    pub author: String,
    pub repo_name: String,
    pub key: String,
    pub branch_to_merge: String,
    pub branch_to_merge_into: String,
    pub pr_number: u64,
    pub repo_owner: String,
}

impl PullRequest {
    pub fn get_audio_path(&self) -> String {
        format!("{}.mp3", self.diff_url)
    }
}

#[derive(Debug, Clone)]
pub struct PullRequestInfo {
    pub pull_request: PullRequest,
    pub left_votes: usize,
    pub right_votes: usize,
}

impl PullRequest {
    async fn new_from_payload(payload: PullRequestEventPayload) -> Self {
        let diff_url = payload.pull_request.diff_url.clone().unwrap();
        let title = payload.pull_request.title.clone().unwrap();
        let additions = payload.pull_request.additions.unwrap();
        let deletions = payload.pull_request.deletions.unwrap();
        let changed_files = payload.pull_request.changed_files.unwrap();
        let author = payload.pull_request.user.unwrap().login.clone();
        let repo_name = payload.pull_request.repo.unwrap();
        let key = payload.pull_request.head.sha.clone();
        let repo_owner = payload.pull_request.base.repo.unwrap().owner.unwrap().login;

        Self {
            diff_url: diff_url.to_string(),
            title: title,
            additions: additions as usize,
            deletions: deletions as usize,
            changed_files: changed_files as usize,
            author: author,
            repo_name: repo_name.name,
            key: key,
            pr_number: payload.pull_request.number,
            branch_to_merge: payload.pull_request.head.label.unwrap(),
            branch_to_merge_into: payload.pull_request.base.label.unwrap(),
            repo_owner,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Server {
    all_prs: Arc<RwLock<HashMap<String, PullRequestInfo>>>,
}

impl Server {
    pub async fn new() -> Self {
        println!("Starting server...");
        let addr = "0.0.0.0:8080";
        let mut router = Router::new();

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router.clone()).await.unwrap();
        let server = Self {
            all_prs: Arc::new(RwLock::new(HashMap::new())),
        };

        server
    }

    async fn webhook_handler(&self, raw_payload: Json<PullRequestEventPayload>) {
        let payload = raw_payload.0;
        let pull_request = PullRequest::new_from_payload(payload.clone()).await;
        self.all_prs.write().unwrap().insert(
            pull_request.diff_url.clone(),
            PullRequestInfo {
                pull_request: pull_request.clone(),
                left_votes: 0,
                right_votes: 0,
            },
        );
    }

    pub fn get_all_prs(&self) -> Vec<PullRequest> {
        let all_prs = self.all_prs.read().unwrap();
        all_prs
            .iter()
            .map(|(_, pr)| pr.pull_request.clone())
            .collect()
    }

    pub fn vote_on_pr(&self, diff_url: String, direction: Direction) {
        let mut all_prs = self.all_prs.write().unwrap();
        if let Some(pr) = all_prs.get_mut(&diff_url) {
            match direction {
                Direction::Left => pr.left_votes += 1,
                Direction::Right => pr.right_votes += 1,
            }
        }
    }

    async fn finalize_vote(&self, diff_url: String) {
        // wait for the vote to be finalized after 30 minutes
        const VOTE_TIME: Duration = Duration::from_secs(60 * 30);
        tokio::time::sleep(VOTE_TIME).await;
        let mut all_prs = self.all_prs.write().unwrap();
        let pr = all_prs.remove(&diff_url);

        if let Some(pr) = pr {
            if pr.left_votes > pr.right_votes {
                // merge the PR
                crate::github_bot::bot::merge(pr).await;
            } else {
                // deny the PR
                crate::github_bot::bot::deny_merge(pr).await;
            }
        }
    }
}

pub enum Direction {
    Left,
    Right,
}
