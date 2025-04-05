use dioxus::prelude::*;

use components::Navbar;
use views::Home;

mod components;
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
    let addr = dioxus::cli_config::fullstack_address_or_localhost();
    let router = axum::Router::new()
        // Server side render the application, serve static assets, and register server functions
        .serve_dioxus_application(ServeConfig::builder(), App)
        .into_make_service();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
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
