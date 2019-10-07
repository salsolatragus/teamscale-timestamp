[![Build Status](https://travis-ci.com/cqse/teamscale-timestamp.svg?branch=master)](https://travis-ci.com/cqse/teamscale-timestamp)

Small command-line utility for build environments to automatically
compute the value for the `?t=` REST request parameter Teamscale
expects when uploading external analysis results (e.g. coverage,
findings, ...)

# Usage

Usage depends on your version control system. Supported VCSs:

- Git
- SVN
- Team Foundation Version Control

Run with `--help` to see all available options and with `--verbose` to see what's going on and
to  debug problems.

## For Git

Simply run the command `teamscale-timestamp` inside your build system's
checkout of your version control repository. If it succeeds, use its
output as the value for the `?t=` parameter in your REST request.

For Git, the branch cannot always be determined from the repository
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
out branch via the `--branch` command line switch:

```sh
teamscale-timestamp.exe --branch develop
```

## For SVN

Simply run the command `teamscale-timestamp` inside your build system's
checkout of your version control repository. If it succeeds, use its
output as the value for the `?t=` parameter in your REST request.

**Note on branches:** Only branches with a single subfolder are supported. E.g. `repo/branches/release1.2` will
work, while `repo/branches/release/1.2` will not. In the latter case, please manually specify the
correct branch via the `--branch` parameter:

```sh
teamscale-timestamp.exe --branch releases/1.2
```

## For Team Foundation Version Control

You must manually specify the branch you are building via the `--branch` parameter as TFS/Azure DevOps builds do not always report the correct branch name.

```sh
teamscale-timestamp.exe --branch Releases/2.56
```

Since the TFVC integration needs to talk to the REST API of the TFS, you must furthermore make an OAuth access token available to the program by activiting `Additional options > Allow scripts to access OAuth token` for
your pipeline job.  This should set the environment variable `SYSTEM_ACCESSTOKEN`.

_Alternatively_, you can provide a personal access token (_not an OAuth access token!_) via the command line option `--tfs-pat`. The user this access token belongs to must be able to read Work Items.

Example invocation with personal access token:

```sh
teamscale-timestamp.exe --tfs-pat arlGrraLLVOL323ara33556 --branch Releases/2.56
```

# Development

Please use IntelliJ for development and configure it to run `rust-fmt` on save.
Under Linux, please install `libssl-dev` to obtain OpenSSL headers. Otherwise your compile may fail.

