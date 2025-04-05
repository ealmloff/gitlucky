use axum::{routing::post, Json, Router};

use dioxus::prelude::{DioxusRouterExt, ServeConfig};
use octocrab::models::events::payload::PullRequestEventPayload;
use std::sync::{Arc, RwLock};

use crate::App;

#[derive(Debug, Clone)]
struct PullRequest {
    diff_url: String,
    title: String,
    additions: usize,
    deletions: usize,
    changed_files: usize,
    author: String,
    repo_name: String,
    key: String,
}

impl PullRequest {
    fn new_from_payload(payload: PullRequestEventPayload) -> Self {
        let diff_url = payload.pull_request.diff_url.clone().unwrap();
        let title = payload.pull_request.title.clone().unwrap();
        let additions = payload.pull_request.additions.unwrap();
        let deletions = payload.pull_request.deletions.unwrap();
        let changed_files = payload.pull_request.changed_files.unwrap();
        let author = payload.pull_request.user.unwrap().login.clone();
        let repo_name = payload.pull_request.repo.unwrap();
        let key = payload.pull_request.head.sha.clone();

        Self {
            diff_url: diff_url.to_string(),
            title: title,
            additions: additions as usize,
            deletions: deletions as usize,
            changed_files: changed_files as usize,
            author: author,
            repo_name: repo_name.name,
            key: key,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Server {
    all_prs: Arc<RwLock<Vec<PullRequest>>>,
}

impl Server {
    pub async fn new() -> Self {
        println!("Starting server...");
        let addr = "0.0.0.0:8080";
        let router = Router::new();

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router.clone()).await.unwrap();
        let server = Self {
            all_prs: Arc::new(RwLock::new(Vec::new())),
        };

        let s_c = server.clone();
        router
            .serve_dioxus_application(ServeConfig::builder(), App)
            .route(
                "/",
                post(move |payload: Json<PullRequestEventPayload>| async move {
                    s_c.webhook_handler(payload).await;
                }),
            );
        server
    }

    async fn webhook_handler(&self, raw_payload: Json<PullRequestEventPayload>) {
        let payload = raw_payload.0;
        let pull_request = PullRequest::new_from_payload(payload.clone());
        self.all_prs.write().unwrap().push(pull_request.clone());
    }
}
