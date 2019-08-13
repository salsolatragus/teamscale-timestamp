use std::option::Option;
use std::string::String;

use crate::git::Git;
use crate::svn::Svn;

pub struct App {
    verbose: bool,
}

impl App {
    pub fn new(verbose: bool) -> App {
        return App { verbose };
    }

    pub fn log(&self, message: &str) {
        if self.verbose {
            println!("{}", message)
        }
    }

    fn guess_branch_from_svn(&self) -> Option<String> {
        let svn = Svn::new(self);
        if svn.is_svn() {
            match svn.branch() {
                Some(branch) => return Some(branch),
                None => (),
            }
        }
        return svn.branch_from_environment();
    }

    fn guess_branch_from_git(&self) -> Option<String> {
        let git = Git::new(self);
        if git.is_git() {
            match git.guess_branch() {
                Some(branch) => return Some(branch),
                None => (),
            }
        }

        return None;
    }

    fn guess_branch_from_environment(&self) -> Option<String> {
        let opt_branch = std::env::var("BRANCH").ok();
        self.log(&format!("$BRANCH={}", opt_branch.clone().unwrap_or("".to_string())));
        let opt_git_branch = std::env::var("GIT_BRANCH").ok();
        self.log(&format!("$GIT_BRANCH={}", opt_git_branch.clone().unwrap_or("".to_string())));
        return opt_git_branch.or(opt_branch);
    }

    pub fn guess_branch(&self) -> Option<String> {
        self.log("Trying to determine branch");
        return self.guess_branch_from_svn()
            .or(self.guess_branch_from_git())
            .or(self.guess_branch_from_environment());
    }

    pub fn guess_timestamp(&self) -> Option<String> {
        self.log("Trying to determine timestamp");
        let svn = Svn::new(self);
        if svn.is_svn() {
            match svn.timestamp() {
                Some(timestamp) => return Some(timestamp),
                None => (),
            }
        }

        let git = Git::new(self);
        if git.is_git() {
            match git.head_timestamp() {
                Some(timestamp) => return Some(timestamp),
                None => (),
            }
        }

        return None;
    }
}