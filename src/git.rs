extern crate regex;

use std::process::Command;

use crate::app::App;
use crate::utils::StrFromOutput;

use self::regex::Regex;

/// Struct for retrieving info from a git repo.
pub struct Git<'a> {
    app: &'a App,
}

impl<'a> Git<'a> {
    pub fn new(app: &'a App) -> Git<'a> {
        return Git { app };
    }

    /// Runs git with the given arguments and returns the result if the git command succeeded.
    fn git(&self, args: &[&str]) -> Option<String> {
        self.app.log(&format!("Running git {}", args.join(" ")));

        let result = Command::new("git")
            .args(args)
            .output()
            .map_to_stdout();

        match &result {
            Err(error) => self.app.log(&format!("git {} failed: {}", args.join(" "), error.to_string())),
            _ => (),
        }

        return result.ok();
    }

    /// Checks if the current directory is part of some Git repo.
    pub fn is_git(&self) -> bool {
        let opt_stdout = self.git(&["rev-parse", "--is-inside-work-tree"]);

        match opt_stdout {
            Some(ref stdout) if stdout.trim().eq("true") => {
                self.app.log("Current directory is in git");
                return true;
            }
            _ => {
                self.app.log("Current directory is not in git");
                return false;
            }
        }
    }

    /// Returns the TS timestamp for the checked out commit.
    pub fn head_timestamp(&self) -> Option<String> {
        return self.git(&["--no-pager", "log", "-n1", "--format=%ct000"]);
    }

    fn extract_single_branch(&self, branch_text: &str) -> Option<String> {
        let lines = branch_text.lines();
        let branch_regex = Regex::new("^\\s*[*]\\s*").unwrap();

        let branches: Vec<String> = lines
            .map(|line| branch_regex.replace_all(line.trim(), "").to_string())
            .filter(|branch| !branch.contains("HEAD detached"))
            .collect();
        self.app.log(&format!("Found the following branches in the Git repo for the HEAD commit: {}", branches.join(", ")));

        if branches.len() != 1 {
            return None;
        }

        return match branches.first() {
            Some(branch) => return Some(branch.to_string()),
            _ => None
        };
    }

    /// Last resort: try to guess the branch from the checked out commit.
    /// Will list all local branches this commit is part of. If there's exactly one,
    /// returns that. Otherwise returns None.
    pub fn guess_branch(&self) -> Option<String> {
        let opt_branches = self.git(&["branch", "--contains"]);
        return opt_branches.and_then(|branch_text| self.extract_single_branch(&branch_text));
    }
}