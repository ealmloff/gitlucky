use dioxus::prelude::*;

#[cfg(not(feature = "server"))]
const ECHO_CSS: Asset = asset!("/assets/styling/echo.css");

/// Echo component that demonstrates fullstack server functions.
#[cfg(not(feature = "server"))]
#[component]
pub fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        document::Link { rel: "stylesheet", href: ECHO_CSS }
        div { id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput: move |event| async move {
                    #[cfg(not(feature = "server"))]
                    let data = echo_server(event.value()).await.unwrap();
                    #[cfg(not(feature = "server"))]
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

#[cfg(not(feature = "server"))]
/// Echo the user input on the server.
#[server(EchoServer)]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}
