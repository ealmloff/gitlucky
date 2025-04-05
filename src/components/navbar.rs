#[cfg(not(feature = "server"))]
use crate::Route;
use dioxus::prelude::*;

#[cfg(not(feature = "server"))]
const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[cfg(not(feature = "server"))]
#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        // div {
        //     id: "navbar",
        //     Link {
        //         to: Route::Home {},
        //         "Home"
        //     }
        // }

        Outlet::<Route> {}
    }
}
