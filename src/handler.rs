use super::config::Target;
use super::queue::Queue;
use super::job::convert_target_to_job;
use serde::{Deserialize, Serialize};
use rocket::State;
use rocket_contrib::json::Json;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusPayload {
    pub sha: String,
    pub name: String,
    pub state: String,
    pub branches: Vec<Branch>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: Commit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
}

// TODO: return enums to match statuses in README
// TODO: General cleanup of code style
pub fn process_payload(
    queue: State<Arc<Queue>>,
    payload: Json<StatusPayload>,
    targets: State<Vec<Target>>,
) -> Option<bool> {
    match &payload.state {
        action if action == "success" => {
            let target_branch =
                determine_hook_branch(payload.sha.clone(), &payload.branches);
            match target_branch {
                Some(branch) => {
                    match targets_for_branch(payload.name.clone(), &branch.name, &targets) {
                        Some(ts) => {
                            for t in ts {
                                let job = convert_target_to_job(t);
                                queue.add(job).unwrap();
                            };

                            Some(true)
                        },
                        None => None,
                    }
                }
                None => None,
            }
        }
        _ => None,
    }
}

fn determine_hook_branch(target_sha: String, branches: &Vec<Branch>) -> Option<&Branch> {
    branches
        .iter()
        .find(|&branch| branch.commit.sha == target_sha)
}

fn targets_for_branch(
    repo: String,
    branch_name: &str,
    targets: &Vec<Target>,
) -> Option<Vec<Target>> {
    let targets_for_repo: Vec<Target> = targets
        .iter()
        .filter(|&target| -> bool {
            let repo_match = target.repository == repo;
            let branch_match = target.branch == branch_name;
            repo_match && branch_match
        })
        .cloned()
        .collect();

    match targets_for_repo.len() {
        0 => None,
        _ => Some(targets_for_repo),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_branch_returns_correct_branch() {
        let mut branches = Vec::new();
        branches.push(Branch {
            name: "master".to_owned(),
            commit: Commit {
                sha: "1234".to_owned(),
            },
        });
        branches.push(Branch {
            name: "develop".to_owned(),
            commit: Commit {
                sha: "5678".to_owned(),
            },
        });

        assert_eq!(
            determine_hook_branch("1234".to_owned(), &branches).unwrap(),
            &branches[0]
        );
    }

    #[test]
    fn determine_branch_returns_correct_none() {
        let mut branches = Vec::new();
        branches.push(Branch {
            name: "master".to_owned(),
            commit: Commit {
                sha: "1234".to_owned(),
            },
        });

        assert_eq!(
            determine_hook_branch("5".to_owned(), &branches).is_none(),
            true
        );
    }
}
