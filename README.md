Small command-line utility for build environments to automatically
compute the value for the `?t=` REST request parameter Teamscale
expects when uploading external analysis results (e.g. coverage,
findings, ...).

# Usage

Simply run the command `teamscale-timestamp` inside your build system's
checkout of your Git or SVN repository. If it succeeds, use its
output as the value for the `?t=` parameter in your REST request.

Run with `--help` to see all available options.

# Supported Build Environments

For SVN repositories, the build environment does not matter (all
required information will be read from the SVN checkout).

For Git, the branch cannot always be determined from the repository
clone alone. Instead, the tool tries to read the checked out branch
from your build tool. The following build environments are supported:

- TeamCity
- TFS and Azure Devops
- Jenkins, but only for plugins that set the `BRANCH` or `GIT_BRANCH`
  environment variable
- CircleCI
- TravisCI
- Appveyor
- Gitlab Pipelines
- Bitbucket Pipelines

If your environment is not supported, you can manually pass the checked
out Git branch via the `--branch` command line switch.
