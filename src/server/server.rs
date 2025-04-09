use axum::{
    routing::{get_service, post},
    Json, Router,
};

// 1 day
const MERGE_MINUTES: u64 = 60 * 24;

#[cfg(not(feature = "server"))]
use dioxus::prelude::{DioxusRouterExt, ServeConfig};
use octocrab::models::{
    events::payload::{PullRequestEventAction, PullRequestEventPayload},
    pulls::PullRequestAction,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    sync::{Arc, RwLock},
    time::Duration,
};

#[cfg(not(feature = "server"))]
use crate::App;
use crate::{Direction, PullRequest};

impl PullRequest {
    pub fn get_audio_path(&self) -> String {
        format!("data/{}.mp3", self.diff_url)
    }
}

#[derive(Debug, Clone)]
pub struct PullRequestInfo {
    pub pull_request: PullRequest,
    pub left_votes: usize,
    pub right_votes: usize,
    pub creation_time: chrono::DateTime<chrono::Utc>,
}

impl PullRequest {
    async fn new_from_payload(payload: PullRequestEventPayload) -> Self {
        //println!("payload: {:?}", payload);
        let diff_url = payload.pull_request.diff_url.clone().unwrap();
        let title = payload.pull_request.title.clone().unwrap();
        let additions = payload.pull_request.additions.unwrap();
        let deletions = payload.pull_request.deletions.unwrap();
        let changed_files = payload.pull_request.changed_files.unwrap();
        let author = payload.pull_request.user.as_ref().unwrap().login.clone();
        println!("Diff url: {}", diff_url);
        let repo_name = diff_url.as_str().split('/').nth(4).unwrap().to_string();
        let key = payload.pull_request.head.sha.clone();
        let repo_owner = &payload
            .pull_request
            .base
            .repo
            .as_ref()
            .unwrap()
            .owner
            .as_ref()
            .unwrap()
            .login;
        let profile_pic_url = payload
            .pull_request
            .user
            .as_ref()
            .unwrap()
            .avatar_url
            .to_string();

        // The heads are something like "owner:branch" so we need to split it and get the branch name
        let head_label = payload.pull_request.head.label.unwrap();
        let branch_to_merge = head_label.split(':').nth(1).unwrap();

        let base_label = payload.pull_request.base.label.unwrap();
        let branch_to_merge_into = base_label.split(':').nth(1).unwrap();
        println!("Branch to merge: {}", branch_to_merge);
        println!("Branch to merge into: {}", branch_to_merge_into);
        let diff = reqwest::get(diff_url.clone())
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let pr = Self {
            diff_url: diff_url.to_string(),
            diff,
            title: title,
            additions: additions as usize,
            deletions: deletions as usize,
            changed_files: changed_files as usize,
            author: author,
            repo_name: repo_name,
            key: Some(key),
            pr_number: payload.pull_request.number,
            branch_to_merge: branch_to_merge.to_string(),
            branch_to_merge_into: branch_to_merge_into.to_string(),
            repo_owner: repo_owner.to_string(),
            profile_pic_url,
        };

        pr
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

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let mut server = Self {
            all_prs: Arc::new(RwLock::new(HashMap::new())),
        };

        let s_c = server.clone();
        let mut router = Router::new().route(
            "/", // The github webhook
            post(move |payload: Json<PullRequestEventPayload>| async move {
                s_c.webhook_handler(payload).await;
            })
            .get_service(tower_http::services::ServeFile::new(
                "target/dx/gitlucky/debug/web/public/index.html",
            )),
        );
        let s_c = server.clone();
        router = router.route(
            "/pr",
            axum::routing::get(move || async move { Json(s_c.get_random_pr()) }),
        );
        let s_c = server.clone();
        router = router.route(
            "/vote",
            post(move |payload: Json<(String, Direction)>| async move {
                let (diff_url, direction) = payload.0;
                s_c.clone().vote_on_pr(diff_url, direction);
            }),
        );
        router = router.fallback_service(
            get_service(
                tower_http::services::ServeDir::new("target/dx/gitlucky/debug/web/public")
                    .append_index_html_on_directories(true),
            )
            .handle_error(|_| async { (axum::http::StatusCode::INTERNAL_SERVER_ERROR, ()) }),
        );

        axum::serve(listener, router).await.unwrap();
        server.load_prs();
        server
    }

    fn load_prs(&self) {
        // Load the prs from the file
        let file = std::fs::File::open("prs.json").unwrap();
        let all_prs: Vec<PullRequestInfo> = serde_json::from_reader(file).unwrap();
        let mut all_prs_map = HashMap::new();
        for pr in all_prs {
            all_prs_map.insert(pr.pull_request.diff_url.clone(), pr);
        }
        let mut all_prs = self.all_prs.write().unwrap();
        all_prs.clear();
        all_prs.extend(all_prs_map);
    }

    async fn webhook_handler(&self, raw_payload: Json<PullRequestEventPayload>) {
        let payload = raw_payload.0;
        if payload.action != PullRequestEventAction::Opened
            && payload.action != PullRequestEventAction::Reopened
        {
            println!("Ignoring action: {:?}", payload.action);
            return;
        }
        if payload.pull_request.mergeable == Some(false) {
            println!("Ignoring unmergeable PR: {:?}", payload.pull_request);
            return;
        }

        let s_c = self.clone();
        let diff_url = payload.pull_request.diff_url.clone().unwrap().to_string();
        let handle = tokio::spawn(async move { s_c.finalize_vote(diff_url, MERGE_MINUTES).await });
        let creation_time = payload
            .pull_request
            .created_at
            .unwrap_or(chrono::Utc::now());
        let pull_request = PullRequest::new_from_payload(payload.clone()).await;
        self.all_prs.write().unwrap().insert(
            pull_request.diff_url.clone(),
            PullRequestInfo {
                pull_request: pull_request.clone(),
                left_votes: 0,
                right_votes: 0,
                creation_time,
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
        println!("Voting on PR: {:?}, {:?}", diff_url, direction);
        let mut all_prs = self.all_prs.write().unwrap();
        if let Some(pr) = all_prs.get_mut(&diff_url) {
            match direction {
                Direction::Left => pr.left_votes += 1,
                Direction::Right => pr.right_votes += 1,
            }
        }
    }

    async fn finalize_vote(&self, diff_url: String, delay_minutes: u64) {
        // wait for the vote to be finalized after a certain amount of time
        let vote_time: Duration = Duration::from_secs(60 * delay_minutes);
        tokio::time::sleep(vote_time).await;
        let pr = {
            let mut all_prs = self.all_prs.write().unwrap();
            all_prs.remove(&diff_url)
        };
        println!("Finalizing vote for PR: {:?}", pr);

        if let Some(pr) = pr {
            if pr.left_votes < pr.right_votes {
                // merge the PR
                println!("Merging PR: {:?}", pr.pull_request);
                crate::github_bot::bot::merge(pr).await;
            } else {
                // deny the PR
                println!("Denying PR: {:?}", pr.pull_request);
                crate::github_bot::bot::deny_merge(pr).await;
            }
        }
    }

    /// Get a random pull request from the list of all pull requests
    /// Sets the key to none so that we don't just publish api keys to the world
    fn get_random_pr(&self) -> PullRequest {
        let all_prs = self.all_prs.read().unwrap();
        let mut rng = rand::thread_rng();
        let random_index = rng.random_range(0..all_prs.len());
        let mut pr = all_prs
            .values()
            .nth(random_index)
            .unwrap()
            .pull_request
            .clone();
        pr.key = None;
        pr
    }
}

// Handle crashes
impl Drop for Server {
    fn drop(&mut self) {
        println!("Server is shutting down...");
        // Save the state of the server to a file
        let all_prs = self.all_prs.read().unwrap();
        let mut file = std::fs::File::create("prs.json").unwrap();
        let all_prs: Vec<PullRequestInfo> = all_prs.values().cloned().collect();
        let all_prs = serde_json::to_string(&all_prs).unwrap();
        file.write_all(all_prs.as_bytes()).unwrap();
        println!("Saved state to prs.json");
    }
}
