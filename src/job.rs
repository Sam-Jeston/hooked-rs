use super::config::Target;

#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub id: String,
    pub command: String,
}

pub fn convert_target_to_job(target: Target) -> Job {
    Job {
        id: format!("{} {}", target.repository, target.branch),
        command: build_command_string(&target),
    }
}

pub fn build_command_string(target: &Target) -> String {
    let base_command = format!("cd {};", target.directory);
    target
        .steps
        .iter()
        .fold(base_command, |acc, comm| format!("{}{};", acc, comm))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_command_string_correctly() {
        let expected_string = format!("cd /var/www;ls stat;echo hello;");
        let target = Target {
            repository: "unused".to_owned(),
            branch: "unused".to_owned(),
            directory: "/var/www".to_owned(),
            steps: vec!["ls stat".to_owned(), "echo hello".to_owned()],
        };

        assert_eq!(build_command_string(&target), expected_string);
    }
}
