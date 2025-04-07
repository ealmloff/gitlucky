use dioxus::html::g;
use octocrab::models::{App, AppId};

use crate::server::server::PullRequestInfo;

fn read_env_var(var_name: &str) -> String {
    let err = format!("Missing environment variable: {var_name}");
    std::env::var(var_name).expect(&err)
}

async fn get_octocrab() -> Option<octocrab::Octocrab> {
    /*let app_id = read_env_var("GITHUB_APP_ID").parse::<u64>().unwrap().into();
    let app_private_key = include_str!("GITHUB_API_KEY.pem").to_string();
    let crab = octocrab::Octocrab::builder()
        .app(
            AppId(app_id),
            jsonwebtoken::EncodingKey::from_rsa_pem(app_private_key.as_bytes()).unwrap(),
        )
        .build()
        .ok();*/
    let crab = octocrab::Octocrab::builder()
        .personal_token(read_env_var("GITHUB_TOKEN"))
        .build()
        .ok();
    crab
}

pub async fn merge(potential_merge: PullRequestInfo) {
    let PullRequestInfo {
        pull_request,
        left_votes,
        right_votes,
    } = potential_merge;

    let token = pull_request.key.clone();
    let people_accepted = right_votes;
    let people_denied = left_votes;
    let branch_to_merge = pull_request.branch_to_merge.clone();
    let branch_to_merge_into = pull_request.branch_to_merge_into.clone();
    let repo_owner = pull_request.repo_owner.clone();
    let repo_name = pull_request.repo_name.clone();
    let maybe_octo = get_octocrab().await;

    if let Some(octocrab) = maybe_octo {
        let _ = octocrab
            .repos(repo_owner, repo_name)
            .merge(&branch_to_merge, branch_to_merge_into)
            .commit_message(format!(
                "The people have merged {}, {} accepted, {} denied.",
                branch_to_merge, people_accepted, people_denied
            ))
            .send()
            .await
            .unwrap();
    } else {
        println!("Error: Octocrab failed to build.");
    }
}

pub async fn deny_merge(potential_merge: PullRequestInfo) {
    let mut should_return = false;
    let pr_number = potential_merge.pull_request.pr_number;
    let token = potential_merge.pull_request.key.clone();
    let people_accepted = potential_merge.right_votes;
    let people_denied = potential_merge.left_votes;
    let repo_owner = potential_merge.pull_request.repo_owner.clone();
    let repo_name = potential_merge.pull_request.repo_name.clone();

    let octocrab = get_octocrab().await.unwrap();

    // Comment on the PR
    let _ = octocrab
        .issues(&repo_owner, &repo_name)
        .create_comment(
            pr_number,
            format!(
                "The people have spoken and have denied this PR! {} accepted, {} denied.",
                people_accepted, people_denied
            ),
        )
        .await
        .unwrap();

    let _ = octocrab
        .pulls(repo_owner, repo_name)
        .update(pr_number)
        .state(octocrab::params::pulls::State::Closed)
        .send()
        .await
        .unwrap();
}
