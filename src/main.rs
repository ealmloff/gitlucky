use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::server::server::Server;
use components::Navbar;
use views::Home;

mod components;
#[cfg(feature = "server")]
mod github_bot;
mod server;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
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

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}
