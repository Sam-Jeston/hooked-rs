use super::builder::Build;

pub struct GithubWebhook<T> {
  action: String,
  payload: T
}

pub struct StatusPayload {
  sha: String,
  name: String,
  state: String,
  branches: Vec<Branch>
}

#[derive(Debug, PartialEq)]
pub struct Branch {
  name: String,
  commit: Commit
}

#[derive(Debug, PartialEq)]
pub struct Commit {
  sha: String
}

pub fn process_payload(payload: GithubWebhook<StatusPayload>, builds: Vec<Build>) -> Option<bool> {
  match payload.action {
    action if action == "status" => {
      Some(true)
    },
    _ => None
  }
}

fn determine_hook_branch(target_sha: String, branches: &Vec<Branch>) -> Option<&Branch> {
  branches.iter().find(|&branch| branch.commit.sha == target_sha)
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
            sha: "1234".to_owned()
          }
        });
        branches.push(Branch {
          name: "develop".to_owned(),
          commit: Commit {
            sha: "5678".to_owned()
          }
        });

        assert_eq!(determine_hook_branch("1234".to_owned(), &branches).unwrap(), &branches[0]);
    }

    #[test]
    fn determine_branch_returns_correct_none() {
        let mut branches = Vec::new();
        branches.push(Branch {
          name: "master".to_owned(),
          commit: Commit {
            sha: "1234".to_owned()
          }
        });

        assert_eq!(determine_hook_branch("5".to_owned(), &branches).is_none(), true);
    }
}