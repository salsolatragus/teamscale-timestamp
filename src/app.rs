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

    fn env_variable(&self, name: &str) -> Option<String> {
        let opt_value = std::env::var(name).ok();
        self.log(&format!("${}={}", name, opt_value.clone().unwrap_or("".to_string())));
        return opt_value;
    }

    fn guess_branch_from_environment(&self) -> Option<String> {
        // common names
        return self.env_variable("BRANCH")
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
            .or(self.env_variable("APPVEYOR_REPO_BRANCH"));
    }

    pub fn guess_branch(&self) -> Option<String> {
        self.log("Trying to determine branch");
        return self.guess_branch_from_svn()
            // since guessing from a git commit is heuristic, we prefer to first check
            // environment variables set by build runners
            .or(self.guess_branch_from_environment())
            .or(self.guess_branch_from_git());
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