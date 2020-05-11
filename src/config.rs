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
    pub port: u16,
    pub host: String,
    pub log: String,
    pub targets: Vec<Target>,
}

pub fn parse_config(yml: String) -> Option<Config> {
    let config: Result<Config, serde_yaml::Error> = serde_yaml::from_str(&yml);
    match config {
        Ok(x) => Some(x),
        _ => None,
    }
}

// TODO: Test
pub fn parse_args<'a>(args: Vec<String>) -> Result<String, &'a str> {
    let config_param = "--config";
    let config_position = args.iter().position(|x| x == config_param);

    match config_position {
        Some(position) if position + 1 >= args.len() => Err("Path not provided for --config"),
        Some(_) if args.len() > 3 => Err("Too many arguments provided"),
        Some(position) => {
            let path_argument = &args[position + 1];
            Ok(path_argument.to_owned())
        }
        None => Err("No config parameter provided"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_yml() {
        let s = "
        log: ./log
        host: localhost
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
