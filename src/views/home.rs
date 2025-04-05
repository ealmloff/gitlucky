use std::{fmt::Display, str::FromStr};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let files = use_server_future(move || async move {
        let response = reqwest::get(
            "https://patch-diff.githubusercontent.com/raw/DioxusLabs/dioxus/pull/3797.diff",
        )
        .await
        .unwrap();
    let text = response.text().await.unwrap();
    GitDiff::from_str(&text).unwrap()
    })
    ?;

    let files = files.read_unchecked();
    let files = files.as_ref().unwrap();

    rsx! {
        pre {
            for file in &files.files {
                div {
                    class: "flex flex-col w-full h-1/2",
                    div {
                        class: "flex flex-row w-full h-4",
                        "{file.old_path} -> {file.new_path}"
                    }
                    div {
                        class: "flex flex-col w-full h-full",
                        for chunk in &file.changes {
                            div {
                                class: "flex flex-row w-full h-4",
                                "{chunk.old_location}->{chunk.new_location}"
                            }
                            div {
                                class: "flex flex-row w-full h-4",
                                "{chunk.context}"
                            }
                            for line in &chunk.contents {
                                pre {
                                    class: if line.status == Status::Added { "color-green-200" },
                                    class: if line.status == Status::Removed { "color-red-200" },
                                    "{line.contents}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
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
                                    else {
                                        panic!("failed to parse 1 {} {}", old.trim_matches('-'), new.trim_matches('+'));
                                    }
                                }
                                else {
                                    panic!("failed to parse 2");
                                }
                            }
                            else {
                                panic!("failed to parse {line}");
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

        Ok(Self {
            files
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GitDiffFile {
    old_path: String,
    new_path: String,
    changes: Vec<GitDiffChange>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GitDiffChange {
    context: String,
    old_location: Location,
    new_location: Location,
    contents: Vec<Line>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Line {
    contents: String,
    status: Status,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
enum Status {
    Added,
    Removed,
    Unchanged,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Location {
    line_number: usize,
    column_number: usize,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:{}", self.line_number, self.column_number)
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
            None => s.parse().map(|line_number| Self {
                line_number,
                column_number:0 
            }).map_err(|_| {})
        }
    }
}
