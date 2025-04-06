use axum::{
    routing::{get_service, post},
    Json, Router,
};

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
    sync::{Arc, RwLock},
    time::Duration,
};

#[cfg(not(feature = "server"))]
use crate::App;
use crate::{ai, Direction, PullRequest};

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

        let pr = Self {
            diff_url: diff_url.to_string(),
            title: title,
            additions: additions as usize,
            deletions: deletions as usize,
            changed_files: changed_files as usize,
            author: author,
            repo_name: repo_name,
            key: Some(key),
            pr_number: payload.pull_request.number,
            branch_to_merge: payload.pull_request.head.label.unwrap(),
            branch_to_merge_into: payload.pull_request.base.label.unwrap(),
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
        let server = Self {
            all_prs: Arc::new(RwLock::new(HashMap::new())),
        };

        let s_c = server.clone();
        let mut router = Router::new().route(
            "/", // The github webhook
            post(move |payload: Json<PullRequestEventPayload>| async move {
                s_c.webhook_handler(payload).await;
            }),
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

        axum::serve(listener, router).await.unwrap();
        server
    }

    async fn webhook_handler(&self, raw_payload: Json<PullRequestEventPayload>) {
        let payload = raw_payload.0;
        if payload.action != PullRequestEventAction::Opened
            || payload.action != PullRequestEventAction::Reopened
        {
            //return;
        }
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
        println!("Voting on PR: {:?}, {:?}", diff_url, direction);
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
        const MERGE_MINUTES: u64 = 1;
        const VOTE_TIME: Duration = Duration::from_secs(60 * MERGE_MINUTES);
        tokio::time::sleep(VOTE_TIME).await;
        let mut all_prs = self.all_prs.write().unwrap();
        let pr = all_prs.remove(&diff_url);

        if let Some(pr) = pr {
            if pr.left_votes > pr.right_votes {
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
