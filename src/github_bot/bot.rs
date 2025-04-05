use octocrab::apps::AppsRequestHandler;
use octocrab::models::Installation;
use octocrab::params::State;

struct Bot {
    token: String,
    octocrab: octocrab::Octocrab,
}

struct PotentialMerge {
    repo_owner: String,
    repo_name: String,
    branch_to_merge_into: String,
    branch_to_merge: String,
}

impl Bot {
    fn new(token: String) -> Self {
        let octocrab = octocrab::Octocrab::builder()
            .personal_token(token.clone())
            .build()
            .unwrap();

        Self { token, octocrab }
    }

    async fn merge(&self, potential_merge: PotentialMerge) {
        let PotentialMerge {
            repo_owner,
            repo_name,
            branch_to_merge_into,
            branch_to_merge,
        } = potential_merge;

        let _ = self
            .octocrab
            .repos(repo_owner, repo_name)
            .merge(&branch_to_merge, branch_to_merge_into)
            .commit_message(format!("The people have merged {}", branch_to_merge))
            .send()
            .await;
    }

    async fn get_all_potential_merges(&self) -> Result<Vec<PotentialMerge>, octocrab::Error> {
        let my_repos = self
            .octocrab
            .current()
            .list_repos_for_authenticated_user()
            .type_("owner")
            .sort("updated")
            .per_page(100)
            .send()
            .await?;

        let mut potential_merges = Vec::new();
        for repo in my_repos {
            let pull_requests = self
                .octocrab
                .pulls(&repo.owner.clone().unwrap().login, &repo.name)
                .list()
                .state(State::Open)
                .send()
                .await?;

            for pull_request in pull_requests {
                potential_merges.push(PotentialMerge {
                    repo_owner: repo.owner.clone().unwrap().login,
                    repo_name: repo.name.clone(),
                    branch_to_merge_into: pull_request.base.label.unwrap(),
                    branch_to_merge: pull_request.head.label.unwrap(),
                });
            }
        }

        Ok(potential_merges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "server")]
    #[tokio::test]
    async fn test_merge() {
        let bot =
            Bot::new(std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required"));
        bot.get_all_potential_merges().await.unwrap();
    }
}
