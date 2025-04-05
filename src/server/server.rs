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
        #[cfg(feature = "dioxus")]
        let addr = dioxus::cli_config::fullstack_address_or_localhost();
        #[cfg(not(feature = "dioxus"))]
        let addr = "0.0.0.0:8080";
        let mut router = Router::new();
        #[cfg(feature = "dioxus")]
        {
            router = router.serve_dioxus_application(ServeConfig::builder(), App);
        }
        #[cfg(not(feature = "dioxus"))]
        {
            router = router.route("/", post(webhook_handler));
        }

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router.clone()).await.unwrap();
        Self { app: router }
    }
}
