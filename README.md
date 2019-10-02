[![Build Status](https://travis-ci.com/cqse/teamscale-timestamp.svg?branch=master)](https://travis-ci.com/cqse/teamscale-timestamp)

Small command-line utility for build environments to automatically
compute the value for the `?t=` REST request parameter Teamscale
expects when uploading external analysis results (e.g. coverage,
findings, ...)

# Usage

Simply run the command `teamscale-timestamp` inside your build system's
checkout of your version control repository. If it succeeds, use its
output as the value for the `?t=` parameter in your REST request.

Run with `--help` to see all available options.

# Supported Version Control Systems

- Git
- SVN
- Team Foundation Version Control

*For SVN*, only branches with a single subfolder are supported. E.g. `repo/branches/release1.2` will
work, while `repo/branches/release/1.2` will not. In the latter case, please manually specify the
correct branch via the `--branch` parameter.

*For Team Foundation Version Control*, you must manually specify the branch you are building via the `--branch` parameter as TFS/Azure DevOps builds do not always report the correct branch name.

# Supported Build Environments

For SVN repositories, the build environment does not matter (all
required information will be read from the SVN checkout).

For Git and TFVC, the branch cannot always be determined from the repository
clone alone. Instead, the tool tries to read the checked out branch
from your build tool. The following build environments are supported:

- TeamCity
- TFS and Azure DevOps
- Jenkins, but only for plugins that set the `BRANCH` or `GIT_BRANCH`
  environment variable
- CircleCI
- TravisCI
- Appveyor
- Gitlab Pipelines
- Bitbucket Pipelines

If your environment is not supported or auto-detection fails, you can manually pass the checked
out branch via the `--branch` command line switch.

# Development

Please use IntelliJ for development and configure it to run `rust-fmt` on save.
Under Linux, please install `libssl-dev` to obtain OpenSSL headers. Otherwise your compile may fail.

