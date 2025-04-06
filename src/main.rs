use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::server::server::Server;
#[cfg(not(feature = "server"))]
use views::Home;

#[cfg(feature = "server")]
mod ai;
#[cfg(feature = "server")]
mod github_bot;
mod server;
mod views;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub diff_url: String,
    pub title: String,
    pub additions: usize,
    pub deletions: usize,
    pub changed_files: usize,
    pub author: String,
    pub repo_name: String,
    pub key: Option<String>,
    pub branch_to_merge: String,
    pub branch_to_merge_into: String,
    pub pr_number: u64,
    pub repo_owner: String,
    pub profile_pic_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Left,
    Right,
}

#[cfg(not(feature = "server"))]
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
}

#[cfg(not(feature = "server"))]
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    use dioxus::logger::tracing::Level;

    println!("Starting server...");
    // let dioxus_logger = dioxus::logger::init(Level::TRACE);
    let server = Server::new().await;
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(not(feature = "server"))]
#[component]
fn App() -> Element {
    rsx! {
        // Global app resources
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}
