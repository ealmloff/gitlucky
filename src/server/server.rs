use axum::{routing::post, Json, Router};

use dioxus::prelude::{DioxusRouterExt, ServeConfig};
use octocrab::models::{events::payload::PullRequestEventPayload, pulls::PullRequestAction};
use serde::Deserialize;

use crate::App;

async fn webhook_handler(raw_payload: Json<PullRequestEventPayload>) {
    println!("Diff: {:?}", raw_payload.0);
}

pub struct Server {
    app: Router,
}

impl Server {
    pub async fn new() -> Self {
        println!("Starting server...");
        let addr = "0.0.0.0:8080";
        let router = Router::new()
            .serve_dioxus_application(ServeConfig::builder(), App)
            .route("/", post(webhook_handler));

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router.clone()).await.unwrap();
        Self { app: router }
    }
}
