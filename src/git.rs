extern crate regex;

use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use crate::app::App;
use crate::utils::StrFromOutput;

use self::regex::Regex;

pub struct Git<'a> {
    app: &'a App,
    directory: Path,
}

impl<'a> Git<'a> {
    fn git(&self, args: &[&str]) -> Option<String> {
        let result = Command::new("git")
            .args(args)
            .output()
            .map_to_stdout();

        match &result {
            Err(error) => self.app.log(&format!("Git command {} failed: {}", args.join(", "), error.to_string())),
            _ => (),
        }

        return result.ok();
    }

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

    pub fn head_timestamp(&self) -> Option<String> {
        return self.git(&["--no-pager", "log", "-n1", "--format=\"%ct000\""]);
    }

    fn extract_single_branch(branch_text: &str) -> Option<String> {
        let lines = branch_text.split_whitespace();
        //lines.map(|line| )
        return None;
    }

    pub fn guess_branch(&self) -> Option<String> {
        let opt_branches = self.git(&["branch", "--contains"]);
        let branch_regex = Regex::new("^");
        return opt_branches.and_then(|branch_text| Git::extract_single_branch(&branch_text));
    }
}