use std::fs::File;
use std::io::Write;
use std::option::Option;
use std::path::Path;
use std::string::String;

use crate::git::Git;
use crate::svn::Svn;
use crate::tfs::Tfs;
use crate::utils::PeekOption;

pub struct App {
    verbose: bool,
    env_reader: fn(&str) -> Option<String>,
}

impl App {
    pub fn new(verbose: bool, env_reader: fn(&str) -> Option<String>) -> App {
        return App {
            verbose,
            env_reader,
        };
    }

    pub fn log(&self, message: &str) {
        if self.verbose {
            println!("{}", message)
        }
    }

    fn branch_from_svn(&self) -> Option<String> {
        self.log("Trying to guess branch name from SVN");
        let svn = Svn::new(self);
        return svn
            .branch()
            .or(svn.branch_from_environment())
            .if_some(|branch| self.log(&format!("Found SVN branch {}", branch)))
            .if_none(|| self.log("Found no SVN branch"));
    }

    fn guess_branch_from_git(&self) -> Option<String> {
        self.log("Trying to guess branch name from Git");
        let git = Git::new(self);
        return git.guess_branch();
    }

    pub fn env_variable(&self, name: &str) -> Option<String> {
        return (self.env_reader)(name).peek_or_default(
            |value| self.log(&format!("${}={}", name, value)),
            "".to_string(),
        );
    }

    fn branch_from_environment(&self) -> Option<String> {
        self.log("Trying to guess branch name from environment variables");
        // common names
        return self
            .env_variable("BRANCH")
            .or(self.env_variable("branch"))
            .or(self.env_variable("GIT_BRANCH"))
            // TeamCity https://stackoverflow.com/questions/13278615/is-there-a-way-to-access-teamcity-system-properties-in-a-powershell-script
            // https://www.jetbrains.com/help/teamcity/predefined-build-parameters.html#PredefinedBuildParameters-Branch-RelatedParameters
            .or(self.env_variable("build_branch"))
            .or(self.env_variable("BUILD_BRANCH"))
            // Jenkins https://github.com/jenkinsci/pipeline-model-definition-plugin/pull/91
            .or(self.env_variable("BRANCH_NAME"))
            // Azure Devops/TFS https://docs.microsoft.com/en-us/azure/devops/pipelines/build/variables?view=azure-devops&tabs=yaml
            .or(self.env_variable("BUILD_SOURCEBRANCHNAME"))
            // Circle CI https://circleci.com/docs/2.0/env-vars/#built-in-environment-variables
            .or(self.env_variable("CIRCLE_BRANCH"))
            // Travis CI https://docs.travis-ci.com/user/environment-variables/#default-environment-variables
            .or(self.env_variable("TRAVIS_BRANCH"))
            // BitBucket pipelines https://confluence.atlassian.com/bitbucket/environment-variables-794502608.html
            .or(self.env_variable("BITBUCKET_BRANCH"))
            // GitLab pipelines https://docs.gitlab.com/ee/ci/variables/predefined_variables.html
            .or(self.env_variable("CI_MERGE_REQUEST_SOURCE_BRANCH_NAME"))
            .or(self.env_variable("CI_COMMIT_REF_NAME"))
            // Appveyor https://www.appveyor.com/docs/environment-variables/
            .or(self.env_variable("APPVEYOR_PULL_REQUEST_HEAD_REPO_BRANCH"))
            .or(self.env_variable("APPVEYOR_REPO_BRANCH"))
            .if_some(|branch| self.log(&format!("Found branch {} in environment", branch)))
            .if_none(|| self.log("Found no branch in environment"));
    }

    pub fn guess_branch(&self) -> Option<String> {
        self.log("Trying to determine branch");
        return self
            .branch_from_svn()
            // since guessing from a git commit is heuristic, we prefer to first check
            // environment variables set by build runners
            .or(self.branch_from_environment())
            .or(self.guess_branch_from_git());
    }

    pub fn guess_timestamp(&self) -> Option<String> {
        self.log("Trying to determine timestamp");
        let svn = Svn::new(self);
        let svn_timestamp = svn
            .timestamp()
            .if_some(|timestamp| self.log(&format!("Found SVN timestamp {}", timestamp)))
            .if_none(|| self.log("Found no SVN timestamp"));

        let git = Git::new(self);
        let git_timestamp = git
            .head_timestamp()
            .if_some(|timestamp| self.log(&format!("Found Git timestamp {}", timestamp)))
            .if_none(|| self.log("Found no Git timestamp"));

        let tfs = Tfs::new(self);
        let tfs_timestamp = tfs
            .timestamp()
            .if_some(|timestamp| self.log(&format!("Found TFVC timestamp {}", timestamp)))
            .if_none(|| self.log("Found no TFVC timestamp"));
        return svn_timestamp.or(git_timestamp).or(tfs_timestamp);
    }

    // TODO comments, print helpful errors in case of e.g. not authenticated, tests for that as well
    // TODO try out in azure devops
    // TODO refactor code for better error handling and logging with results
    // TODO support tfs access token as well? would allow testing!
    // TODO documentation!
    /// Attempts to write revision.txt content to the given file path.
    pub fn write_revision_txt(&self, t: &str, revision_txt_file: &Path) -> std::io::Result<()> {
        let mut file = File::create(revision_txt_file)?;
        file.write_all(format!("timestamp: {}", t).as_ref())?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_environment_means_no_branch() {
        let branch = App::new(true, |_| None).branch_from_environment();
        assert_eq!(None, branch);
    }

    #[test]
    fn read_branch_from_env_variable() {
        let branch = App::new(true, |variable| {
            if variable == "GIT_BRANCH" {
                return Some("the-branch".to_string());
            }
            return None;
        })
        .branch_from_environment();
        assert_eq!(Some("the-branch".to_string()), branch);
    }
}
