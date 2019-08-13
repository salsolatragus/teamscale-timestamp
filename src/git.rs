use std::path::Path;
use std::process::Command;
use crate::utils::StrFromOutput;
use crate::app::App;

pub struct Git<'a> {
    app: &'a App,
    directory: Path,
}

impl<'a> Git<'a> {
    pub fn is_git(&self) -> bool {
        let opt_stdout = Command::new("git")
            .args(&["rev-parse", "--is-inside-work-tree"])
            .output()
            .map_to_stdout();

        match opt_stdout {
            Ok(ref stdout) if stdout.trim().eq("true") => {
                self.app.log("Current directory is in git");
                return true
            },
            _ => {
                self.app.log("Current directory is not in git");
                return false;
            }
        }
    }

    fn head_timestamp(&self) -> Option<String> {
        return None;
    }
}