use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Build {
  repository: String,
  branches: Vec<String>,
  build: Vec<String>,
}

pub fn parse_builds(yml: String) -> Option<Vec<Build>> {
    let jobs: Result<Vec<Build>, serde_yaml::Error> = serde_yaml::from_str(&yml);
    match jobs {
      Ok(x) => Some(x),
      _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_yml() {
        let s =
        "
        - repository: Sam-Jeston/hooked-rs
          branches:
            - master
            - develop
          build:
            - echo \"hello world\"
        ";

        let mut branches = Vec::new();
        branches.push("master".to_owned());
        branches.push("develop".to_owned());
        assert_eq!(parse_builds(s.to_owned()).unwrap()[0].repository, "Sam-Jeston/hooked-rs");
        assert_eq!(parse_builds(s.to_owned()).unwrap()[0].branches, branches);
    }

    #[test]
    fn parse_invalid_yml() {
        let s = "key: value";
        assert_eq!(parse_builds(s.to_owned()).is_none(), true);
    }
}