extern crate regex;

use crate::logger::Logger;
use crate::utils::run;

use self::regex::Regex;

/// Struct for retrieving info from a git repo.
pub struct Git<'a> {
    logger: &'a Logger,
}

impl<'a> Git<'a> {
    pub fn new(logger: &'a Logger) -> Git<'a> {
        return Git { logger };
    }

    /// Runs git with the given arguments and returns the result if the git command succeeded.
    fn git(&self, args: &[&str]) -> Option<String> {
        self.logger.log(&format!("Running git {}", args.join(" ")));
        return match run("git", args, |command| command) {
            Ok(stdout) => Some(stdout),
            Err(error) => {
                self.logger.log(&error);
                None
            }
        };
    }

    /// Checks if the current directory is part of some Git repo.
    fn is_git(&self) -> bool {
        let opt_stdout = self.git(&["rev-parse", "--is-inside-work-tree"]);

        match opt_stdout {
            Some(ref stdout) if stdout.trim().eq("true") => {
                self.logger.log("Current directory is in git");
                return true;
            }
            _ => {
                self.logger.log("Current directory is not in git");
                return false;
            }
        }
    }

    /// Returns the TS timestamp for the checked out commit.
    pub fn head_timestamp(&self) -> Option<String> {
        if !self.is_git() {
            return None;
        }
        return self.git(&["--no-pager", "log", "-n1", "--format=%ct000"]);
    }

    fn preprocess_branch_text(branch_text: &str) -> Vec<String> {
        let lines = branch_text.lines();
        let branch_regex = Regex::new("^\\s*[*]\\s*").unwrap();

        return lines
            .map(|line| branch_regex.replace_all(line.trim(), "").to_string())
            .filter(|branch| !branch.contains("HEAD detached"))
            .collect();
    }

    fn extract_single_branch(&self, branch_text: &str) -> Option<String> {
        let branches = Git::preprocess_branch_text(branch_text);
        match branches.len() {
            0 => {
                self.logger
                    .log("Found no branches in the Git repo that contain the HEAD commit");
                return None;
            }
            1 => {
                self.logger.log(&format!(
                    "Found exactly one branch in the Git repo that contains the HEAD commit: {}",
                    branches.first().unwrap()
                ));
                return branches.first().map(|branch| branch.to_string());
            }
            _ => {
                self.logger.log(&format!(
                    "Found more than one branch in the Git repo that contains the HEAD commit: {}",
                    branches.join(", ")
                ));
                return None;
            }
        }
    }

    /// Last resort: try to guess the branch from the checked out commit.
    /// Will list all local branches this commit is part of. If there's exactly one,
    /// returns that. Otherwise returns None.
    pub fn guess_branch(&self) -> Option<String> {
        if !self.is_git() {
            return None;
        }
        let opt_branches = self.git(&["branch", "--contains"]);
        return opt_branches.and_then(|branch_text| self.extract_single_branch(&branch_text));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_branch_text() {
        assert_eq!(["master"], Git::preprocess_branch_text("master").as_slice());
        assert_eq!(
            ["master"],
            Git::preprocess_branch_text("* master").as_slice()
        );
        assert_eq!(
            ["master", "branch"],
            Git::preprocess_branch_text("* master\nbranch").as_slice()
        );
        assert_eq!(
            ["master"],
            Git::preprocess_branch_text("* (HEAD detached at 6f9a90e36e6)\nmaster\n").as_slice()
        );
    }
}
