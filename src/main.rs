use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::server::server::Server;
#[cfg(not(feature = "server"))]
use views::Home;

#[cfg(feature = "server")]
mod ai;
mod components;
#[cfg(feature = "server")]
mod github_bot;
mod server;
mod views;

#[cfg(not(feature = "server"))]
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
}

#[cfg(not(feature = "server"))]
const FAVICON: Asset = asset!("/assets/favicon.ico");
#[cfg(not(feature = "server"))]
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    use dioxus::logger::tracing::Level;

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
    // Build cool things 🦧🦧🦧

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}
