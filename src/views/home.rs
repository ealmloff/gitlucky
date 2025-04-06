use dioxus::prelude::*;
use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy, PartialEq)]
enum TransitioningDirection {
    Left,
    Right,
}

#[component]
pub fn Home() -> Element {
    let mut transitioning = use_signal(|| None);
    let data = use_resource(move || async move {
        let response = reqwest::get("https://github.com/floneum/floneum/pull/337.diff")
            .await
            .unwrap();
        let text = response.text().await.unwrap();
        let diff = GitDiff::from_str(&text).unwrap();
        PRData {
            repo: "floneum".to_string(),
            pull_request_title: "Add support for `git diff`".to_string(),
            user: "Bob".to_string(),
            user_avatar: "https://avatars.githubusercontent.com/u/123456?v=4".to_string(),
            diff,
        }
    })
    .suspend()?
    .read_unchecked();

    rsx! {
        div { class: "absolute flex flex-col w-[100vw] h-[100vh] max-h-[100vh]",
            onclick: move |evt| async move {
                let pos = evt.client_coordinates();
                if transitioning().is_some() {
                    return;
                }
                let screen_width: f64 = document::eval("return window.innerWidth").join().await.unwrap();
                transitioning.set(Some(if dbg!(pos.x) < dbg!(screen_width) / 2. {
                    TransitioningDirection::Left
                } else {
                    TransitioningDirection::Right
                }));
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                transitioning.set(None);
            },
            div { class: "absolute flex flex-row w-[100vw]",
                div {
                    class: "text-left w-[50vw] p-8",
                    "⬅️ reject"
                }
                div {
                    class: "text-right w-[50vw] p-8",
                    "accept ➡️"
                }
            }
            div {
                class: "ml-[10vw] mt-[10vh]",
                for card in (0..3 + transitioning().is_some() as usize).rev() {
                    Card {
                        key: "{card}",
                        class: if let Some(dir) = transitioning() { if card == 2 + transitioning().is_some() as usize {
                            "in-card"
                        } else if card == 0 {
                            match dir {
                                TransitioningDirection::Right => "right-card",
                                TransitioningDirection::Left => "left-card",
                            }
                        } else {
                            "down-card"
                        } } else { "card" },
                        data: data.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn Card(class: String, data: PRData) -> Element {
    const VIDEO: Asset = asset!("/assets/minecraft.webm", AssetOptions::Unknown);
    let files = data.diff;
    let title = data.pull_request_title;
    let user = data.user;
    let user_avatar = data.user_avatar;

    rsx! {
        div { class: "w-[80vw] h-[80vh] {class}",
            div { class: "absolute flex flex-col w-[80vw] h-[80vh] font-mono",
                video {
                    autoplay: true,
                    r#loop: true,
                    muted: true,
                    class: "w-[80vw] h-full object-cover rounded-xl",
                    source { src: VIDEO, r#type: "video/webm" }
                }
            }
            div { class: "absolute w-[80vw] h-[80vh] font-mono overflow-y-scroll border rounded-xl",
                div { class: "flex flex-col backdrop-blur-xs bg-[rgba(255,255,255,0.5)]",
                    for file in &files.files {
                        div { class: "flex flex-row w-[80vw] font-bold pl-8 sticky h-[25px] top-0 overflow-ellipsis overflow-clip bg-[rgba(195,195,195)]",
                            "{file.old_path} -> {file.new_path}"
                        }
                        div { class: "flex flex-col w-[80vw]",
                            for chunk in &file.changes {
                                div { class: "flex flex-row w-[80vw] border-b pl-8 sticky top-[25px] h-[25px] overflow-ellipsis overflow-clip bg-[rgba(195,195,195)]",
                                    "{chunk.old_location} -> {chunk.new_location} @@ {chunk.context}"
                                }
                                for line in &chunk.contents {
                                    match line.status {
                                        Status::Added => rsx! {
                                            pre { class: "whitespace-pre truncate bg-[rgba(200,255,200,.8)]",
                                                span { class: "p-2", "+" }
                                                "{line.contents}"
                                            }
                                        },
                                        Status::Removed => rsx! {
                                            pre { class: "whitespace-pre truncate bg-[rgba(255,200,200,.8)]",
                                                span { class: "p-2", "-" }
                                                "{line.contents}"
                                            }
                                        },
                                        Status::Unchanged => rsx! {
                                            pre { class: "whitespace-pre truncate",
                                                span { class: "p-2", " " }
                                                "{line.contents}"
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
struct PRData {
    repo: String,
    pull_request_title: String,
    user: String,
    user_avatar: String,
    diff: GitDiff,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
struct GitDiff {
    files: Vec<GitDiffFile>,
}

impl FromStr for GitDiff {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut files = Vec::new();
        let mut lines = s.split_inclusive("\n").peekable();

        while let Some(line) = lines.next() {
            if let Some(from_file) = line.strip_prefix("---") {
                let line = lines.next();
                if let Some(to_file) = line.and_then(|line| line.strip_prefix("+++")) {
                    let old_path = from_file.trim().to_string();
                    let new_path = to_file.trim().to_string();

                    let mut changes = Vec::new();
                    while let Some(line) = lines.next_if(|line| !line.starts_with("---")) {
                        if let Some(line) = line.strip_prefix("@@ ") {
                            if let Some((location, context)) = line.split_once("@@") {
                                if let Some((old, new)) = location.split_once(" ") {
                                    if let (Ok(old_location), Ok(new_location)) = (
                                        old.trim_matches('-').parse(),
                                        new.trim_matches('+').parse(),
                                    ) {
                                        changes.push(GitDiffChange {
                                            context: context.trim().to_string(),
                                            old_location,
                                            new_location,
                                            contents: Vec::new(),
                                        });
                                    }
                                }
                            }
                        } else if let Some(line) = line.strip_prefix("+") {
                            changes.last_mut().unwrap().contents.push(Line {
                                contents: line.to_string(),
                                status: Status::Added,
                            })
                        } else if let Some(line) = line.strip_prefix("-") {
                            changes.last_mut().unwrap().contents.push(Line {
                                contents: line.to_string(),
                                status: Status::Removed,
                            })
                        } else if let Some(line) = line.strip_prefix(" ") {
                            changes.last_mut().unwrap().contents.push(Line {
                                contents: line.to_string(),
                                status: Status::Unchanged,
                            })
                        }
                    }

                    files.push(GitDiffFile {
                        old_path,
                        new_path,
                        changes,
                    });
                }
            }
        }

        Ok(Self { files })
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
struct GitDiffFile {
    old_path: String,
    new_path: String,
    changes: Vec<GitDiffChange>,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
struct GitDiffChange {
    context: String,
    old_location: Location,
    new_location: Location,
    contents: Vec<Line>,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
struct Line {
    contents: String,
    status: Status,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
enum Status {
    Added,
    Removed,
    Unchanged,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
struct Location {
    line_number: usize,
    column_number: usize,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line_number, self.column_number)
    }
}

impl FromStr for Location {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        match s.split_once(',') {
            Some((line, column)) => line
                .parse()
                .and_then(|line_number| {
                    column.parse().map(|column_number| Self {
                        line_number,
                        column_number,
                    })
                })
                .map_err(|_| {}),
            None => s
                .parse()
                .map(|line_number| Self {
                    line_number,
                    column_number: 0,
                })
                .map_err(|_| {}),
        }
    }
}
