use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub repository: String,
    pub branch: String,
    pub directory: String,
    pub steps: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub port: i32,
    pub targets: Vec<Target>,
}

pub fn parse_config(yml: String) -> Option<Config> {
    let config: Result<Config, serde_yaml::Error> = serde_yaml::from_str(&yml);
    match config {
        Ok(x) => Some(x),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_yml() {
        let s = "
        port: 8000
        targets: 
          - repository: Sam-Jeston/hooked-rs
            branch: master
            directory: /var/www
            steps:
              - echo \"hello world\"
        ";

        assert_eq!(
            parse_config(s.to_owned()).unwrap().targets[0].repository,
            "Sam-Jeston/hooked-rs"
        );
    }

    #[test]
    fn parse_invalid_yml() {
        let s = "key: value";
        assert_eq!(parse_config(s.to_owned()).is_none(), true);
    }
}
