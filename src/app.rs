use std::fs::File;
use std::io::Write;
use std::option::Option;
use std::path::Path;
use std::string::String;

use crate::env_reader::EnvReader;
use crate::git::Git;
use crate::logger::Logger;
use crate::svn::Svn;
use crate::tfs::Tfs;
use crate::utils::PeekOption;

pub struct App<'a> {
    logger: &'a Logger,
    env_reader: EnvReader<'a>,
    tfs_personal_access_token: Option<&'a str>,
}

impl<'a> App<'a> {
    pub fn new(
        logger: &'a Logger,
        env_reader: EnvReader<'a>,
        tfs_personal_access_token: Option<&'a str>,
    ) -> App<'a> {
        return App {
            logger,
            env_reader: EnvReader::new(move |name| {
                env_reader.env_variable(name).peek_or_default(
                    |value| logger.log(&format!("${}={}", name, value)),
                    "".to_string(),
                )
            }),
            tfs_personal_access_token,
        };
    }

    fn branch_from_svn(&self) -> Option<String> {
        self.logger.log("Trying to guess branch name from SVN");
        let svn = Svn::new(self.logger);
        return svn
            .branch()
            .or(svn.branch_from_environment())
            .if_some(|branch| self.logger.log(&format!("Found SVN branch {}", branch)))
            .if_none(|| self.logger.log("Found no SVN branch"));
    }

    fn guess_branch_from_git(&self) -> Option<String> {
        self.logger.log("Trying to guess branch name from Git");
        let git = Git::new(self.logger);
        return git.guess_branch();
    }

    fn branch_from_environment(&self) -> Option<String> {
        self.logger
            .log("Trying to guess branch name from environment variables");
        // common names
        return self
            .env_reader
            .env_variable("BRANCH")
            .or(self.env_reader.env_variable("branch"))
            .or(self.env_reader.env_variable("GIT_BRANCH"))
            // TeamCity https://stackoverflow.com/questions/13278615/is-there-a-way-to-access-teamcity-system-properties-in-a-powershell-script
            // https://www.jetbrains.com/help/teamcity/predefined-build-parameters.html#PredefinedBuildParameters-Branch-RelatedParameters
            .or(self.env_reader.env_variable("build_branch"))
            .or(self.env_reader.env_variable("BUILD_BRANCH"))
            // Jenkins https://github.com/jenkinsci/pipeline-model-definition-plugin/pull/91
            .or(self.env_reader.env_variable("BRANCH_NAME"))
            // Azure Devops/TFS https://docs.microsoft.com/en-us/azure/devops/pipelines/build/variables?view=azure-devops&tabs=yaml
            .or(self.env_reader.env_variable("BUILD_SOURCEBRANCHNAME"))
            // Circle CI https://circleci.com/docs/2.0/env-vars/#built-in-environment-variables
            .or(self.env_reader.env_variable("CIRCLE_BRANCH"))
            // Travis CI https://docs.travis-ci.com/user/environment-variables/#default-environment-variables
            .or(self.env_reader.env_variable("TRAVIS_BRANCH"))
            // BitBucket pipelines https://confluence.atlassian.com/bitbucket/environment-variables-794502608.html
            .or(self.env_reader.env_variable("BITBUCKET_BRANCH"))
            // GitLab pipelines https://docs.gitlab.com/ee/ci/variables/predefined_variables.html
            .or(self
                .env_reader
                .env_variable("CI_MERGE_REQUEST_SOURCE_BRANCH_NAME"))
            .or(self.env_reader.env_variable("CI_COMMIT_REF_NAME"))
            // Appveyor https://www.appveyor.com/docs/environment-variables/
            .or(self
                .env_reader
                .env_variable("APPVEYOR_PULL_REQUEST_HEAD_REPO_BRANCH"))
            .or(self.env_reader.env_variable("APPVEYOR_REPO_BRANCH"))
            .if_some(|branch| {
                self.logger
                    .log(&format!("Found branch {} in environment", branch))
            })
            .if_none(|| self.logger.log("Found no branch in environment"));
    }

    pub fn guess_branch(&self) -> Option<String> {
        self.logger.log("Trying to determine branch");
        return self
            .branch_from_svn()
            // since guessing from a git commit is heuristic, we prefer to first check
            // environment variables set by build runners
            .or(self.branch_from_environment())
            .or(self.guess_branch_from_git());
    }

    pub fn guess_timestamp(&self) -> Option<String> {
        self.logger.log("Trying to determine timestamp");
        let svn = Svn::new(self.logger);
        let svn_timestamp = svn
            .timestamp()
            .if_some(|timestamp| {
                self.logger
                    .log(&format!("Found SVN timestamp {}", timestamp))
            })
            .if_none(|| self.logger.log("Found no SVN timestamp"));

        let git = Git::new(self.logger);
        let git_timestamp = git
            .head_timestamp()
            .if_some(|timestamp| {
                self.logger
                    .log(&format!("Found Git timestamp {}", timestamp))
            })
            .if_none(|| self.logger.log("Found no Git timestamp"));

        let tfs = Tfs::new(self.logger, &self.env_reader);
        let tfs_timestamp = tfs
            .timestamp(self.tfs_personal_access_token)
            .if_some(|timestamp| {
                self.logger
                    .log(&format!("Found TFVC timestamp {}", timestamp))
            })
            .if_none(|| self.logger.log("Found no TFVC timestamp"));
        return svn_timestamp.or(git_timestamp).or(tfs_timestamp);
    }

    /// Attempts to write revision.txt content to the given file path.
    pub fn write_revision_txt(&self, t: &str, revision_txt_file: &Path) -> std::io::Result<()> {
        let mut file = File::create(revision_txt_file)?;
        file.write_all(format!("timestamp: {}", t).as_ref())?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::env_reader::EnvReader;

    use super::*;

    #[test]
    fn empty_environment_means_no_branch() {
        let branch =
            App::new(&Logger::new(true), EnvReader::new(|_| None), None).branch_from_environment();
        assert_eq!(None, branch);
    }

    #[test]
    fn read_branch_from_env_variable() {
        let branch = App::new(
            &Logger::new(true),
            EnvReader::new(|variable| {
                if variable == "GIT_BRANCH" {
                    return Some("the-branch".to_string());
                }
                return None;
            }),
            None,
        )
        .branch_from_environment();
        assert_eq!(Some("the-branch".to_string()), branch);
    }
}
