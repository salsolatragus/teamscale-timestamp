extern crate chrono;
extern crate regex;

use std::process::Command;

use crate::app::App;

use self::chrono::DateTime;
use self::regex::Regex;

/// Struct for retrieving info from an SVN repo.
pub struct Svn<'a> {
    app: &'a App,
}

impl<'a> Svn<'a> {
    pub fn new(app: &'a App) -> Svn<'a> {
        return Svn { app };
    }

    /// Runs SVN with the given arguments and returns the result if the command succeeded.
    fn svn(&self, args: &[&str]) -> Option<String> {
        self.app.log(&format!("Running svn {}", args.join(" ")));

        let opt_output = Command::new("svn")
            .env("LANG", "C")
            .args(args)
            .output();

        match opt_output {
            Ok(output) => {
                if !output.status.success() {
                    self.app.log(&format!("svn {} failed with exit code {}", args.join(" "),
                                          output.status.code().unwrap_or(-999)));
                    return None;
                }
                return Some(std::str::from_utf8(output.stdout.as_ref()).unwrap().to_string());
            }
            Err(error) => {
                self.app.log(&format!("svn {} failed: {}", args.join(" "), error.to_string()));
                return None;
            }
        }
    }

    /// Checks if the current directory is part of some SVN repo.
    fn is_svn(&self) -> bool {
        let opt_stdout = self.svn(&["info"]);

        match opt_stdout {
            Some(ref stdout) if stdout.contains("URL:") => {
                self.app.log("Current directory is in SVN");
                return true;
            }
            _ => {
                self.app.log("Current directory is not in SVN");
                return false;
            }
        }
    }

    /// Returns the TS timestamp for the currently checked out revision.
    pub fn timestamp(&self) -> Option<String> {
        if !self.is_svn() {
            return None;
        }
        let opt_date_string = self.svn(&["info", "--show-item", "last-changed-date"]);
        let opt_date = opt_date_string.and_then(|date_string| DateTime::parse_from_rfc3339(&date_string).ok());
        return opt_date.map(|date| format!("{}000", date.timestamp()));
    }

    fn extract_branch_from_url(&self, url: &str) -> Option<String> {
        self.app.log(&format!("Trying to parse SVN URL: {}", url));
        let regex = Regex::new("(?:branches|tags)/(?P<branch>[^/]+)|(?P<trunk>trunk)").unwrap();
        let captures = regex.captures(url)?;
        return match captures.name("branch") {
            Some(capture) => Some(capture.as_str().to_string()),
            _ => match captures.name("trunk") {
                Some(_) => Some("trunk".to_string()),
                _ => None,
            },
        };
    }

    /// Tries to read the SVN branch form environment variables.
    pub fn branch_from_environment(&self) -> Option<String> {
        let environment_result = std::env::var("SVN_URL");
        self.app.log(&format!("$SVN_URL={}", environment_result.clone().unwrap_or("".to_string())));
        return environment_result.ok().and_then(|url| self.extract_branch_from_url(&url));
    }

    /// Extracts the branch from the SVN URL of the current directory.
    pub fn branch(&self) -> Option<String> {
        if !self.is_svn() {
            return None;
        }
        let opt_url = self.svn(&["info", "--show-item", "url"]);
        return opt_url.and_then(|url| self.extract_branch_from_url(&url));
    }
}