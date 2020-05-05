use super::config::Target;
use super::job::convert_target_to_job;
use super::queue::Queue;
use rocket::State;
use rocket_contrib::json::Json;
use rocket::http::Status;
use serde::{Deserialize, Serialize};
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

pub fn process_payload(
    queue: State<Arc<Queue>>,
    payload: Json<StatusPayload>,
    targets: State<Vec<Target>>,
) -> Status {
    let target_branch = determine_hook_branch(payload.sha.clone(), &payload.branches);
    match target_branch {
        Some(branch) => {
            match repo_found(payload.name.clone(), &targets) {
                Some(_) => {
                    let targets = targets_for_branch(payload.name.clone(), &branch.name, &targets);
                    match targets.len() {
                        0 => Status::NoContent,
                        _ if &payload.state != "success"  => Status::Accepted,
                        _ => {
                            for t in targets {
                                let job = convert_target_to_job(t);
                                queue.add(job).unwrap();
                            }

                            Status::Ok
                        }
                    }
                },
                None => Status::NotFound
            }
        },
        // If we don't match the hook branch, then this is a logic failure, hence the 500
        None => Status::InternalServerError
    } 
}

fn determine_hook_branch(target_sha: String, branches: &Vec<Branch>) -> Option<&Branch> {
    branches
        .iter()
        .find(|&branch| branch.commit.sha == target_sha)
}

fn repo_found(
    repo: String,
    targets: &Vec<Target>,
) -> Option<&Target> {
    targets
        .iter()
        .find(|&target| target.repository == repo)
}

fn targets_for_branch(
    repo: String,
    branch_name: &str,
    targets: &Vec<Target>,
) -> Vec<Target> {
    targets
        .iter()
        .filter(|&target| -> bool {
            let repo_match = target.repository == repo;
            let branch_match = target.branch == branch_name;
            repo_match && branch_match
        })
        .cloned()
        .collect()
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
