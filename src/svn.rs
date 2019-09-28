extern crate chrono;
extern crate regex;

use crate::logger::Logger;
use crate::utils::run;

use self::chrono::DateTime;
use self::regex::Regex;

/// Struct for retrieving info from an SVN repo.
pub struct Svn<'a, T: Logger> {
    logger: &'a T,
}

// TODO (FS) doesn't work for e.g. release/2.6 etc. document using --branch
fn extract_branch_from_url(url: &str) -> Option<String> {
    let regex = Regex::new("/(branches|tags)/(?P<branch>[^/]+)|/(?P<trunk>trunk)(/|$)").unwrap();
    let captures = regex.captures(url)?;
    return match captures.name("branch") {
        Some(capture) => Some(capture.as_str().to_string()),
        _ => match captures.name("trunk") {
            Some(_) => Some("trunk".to_string()),
            _ => None,
        },
    };
}

fn svn_date_string_to_timestamp(date_string: &str) -> Option<i64> {
    return DateTime::parse_from_rfc3339(date_string)
        .map(|date| date.timestamp())
        .ok();
}

impl<'a, T: Logger> Svn<'a, T> {
    pub fn new(logger: &'a T) -> Svn<'a, T> {
        return Svn { logger };
    }

    /// Runs SVN with the given arguments and returns the result if the command succeeded.
    fn svn(&self, args: &[&str]) -> Option<String> {
        self.logger.log(&format!("Running svn {}", args.join(" ")));
        return match run("svn", args, |command| command.env("LANG", "C")) {
            Ok(stdout) => Some(stdout),
            Err(error) => {
                self.logger.log(&error);
                None
            }
        };
    }

    /// Checks if the current directory is part of some SVN repo.
    fn is_svn(&self) -> bool {
        let opt_stdout = self.svn(&["info"]);

        match opt_stdout {
            Some(ref stdout) if stdout.contains("URL:") => {
                self.logger.log("Current directory is in SVN");
                return true;
            }
            _ => {
                self.logger.log("Current directory is not in SVN");
                return false;
            }
        }
    }

    /// Returns the TS timestamp for the currently checked out revision.
    pub fn timestamp(&self) -> Option<String> {
        if !self.is_svn() {
            return None;
        }
        let opt_date_string = self
            .svn(&["info", "--show-item", "last-changed-date"])
            .map(|string| string.trim().to_string());
        return match opt_date_string {
            Some(ref date_string) => {
                self.logger
                    .log(&format!("Read date {} from SVN", date_string));
                let timestamp = opt_date_string
                    .and_then(|date_string| svn_date_string_to_timestamp(&date_string));
                return timestamp.map(|timestamp| format!("{}000", timestamp));
            }
            None => None,
        };
    }

    /// Tries to read the SVN branch form environment variables.
    pub fn branch_from_environment(&self) -> Option<String> {
        let environment_result = std::env::var("SVN_URL");
        self.logger.log(&format!(
            "$SVN_URL={}",
            environment_result.clone().unwrap_or("".to_string())
        ));
        return environment_result
            .ok()
            .and_then(|url| extract_branch_from_url(&url));
    }

    /// Extracts the branch from the SVN URL of the current directory.
    pub fn branch(&self) -> Option<String> {
        if !self.is_svn() {
            return None;
        }
        let opt_url = self.svn(&["info", "--show-item", "url"]);
        return match opt_url {
            Some(url) => {
                self.logger
                    .log(&format!("Trying to parse SVN URL: {}", url));
                return extract_branch_from_url(&url);
            }
            None => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_branch_from_url() {
        assert_eq!(
            Some("trunk".to_string()),
            extract_branch_from_url("https://svn.com/repo/trunk/something")
        );
        assert_eq!(
            Some("trunk".to_string()),
            extract_branch_from_url("https://svn.com/repo/trunk")
        );
        assert_eq!(
            Some("foo".to_string()),
            extract_branch_from_url("https://svn.com/repo/branches/foo/something")
        );
        assert_eq!(
            Some("bar".to_string()),
            extract_branch_from_url("https://svn.com/repo/tags/bar/")
        );
        assert_eq!(
            None,
            extract_branch_from_url("https://svn.com/repo/unrelated")
        );
        assert_eq!(
            None,
            extract_branch_from_url("https://svn.com/repo/trunkate")
        );
        assert_eq!(
            None,
            extract_branch_from_url("https://svn.com/repo/branching/blue")
        );
    }

    #[test]
    fn test_date_parsing() {
        assert_eq!(
            Some(1565100814),
            svn_date_string_to_timestamp("2019-08-06T14:13:34.879966Z")
        );
    }
}
