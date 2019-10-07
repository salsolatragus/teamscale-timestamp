extern crate clap;

use std::path::Path;

use clap::Arg;

use crate::app::App;
use crate::env_reader::EnvReader;
use crate::logger::Logger;

mod app;
mod env_reader;
mod git;
mod logger;
mod svn;
mod tfs;
mod utils;

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let matches = clap::App::new("teamscale-timestamp")
        .version(version)
        .about("Tries to determine the value for the ?t= parameter when uploading external data to Teamscale. \
            Run this command from within the working directory of your version control system checkout. \
            Prints the value to be used for the ?t= parameter to STDOUT and exits with exit code 0. \
            If the timestamp cannot be determined, exits with a non-0 exit code.")
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Enable verbose output. Use this to debug what the tool is doing"))
        .arg(Arg::with_name("branch")
            .short("b")
            .long("branch")
            .takes_value(true)
            .value_name("BRANCH")
            .help("Pass a branch name to use for the upload (e.g. master or Main). \
                Use this if automatic detection of the branch does not work"))
        .arg(Arg::with_name("revfile")
            .short("r")
            .long("revision-txt")
            .takes_value(true)
            .value_name("FILE")
            .help("If this option is set, instead of printing to STDOUT, writes the timestamp to \
                the given FILE so it can be used by profilers to map coverage data to the correct \
                code branch + timestamp."))
        .arg(Arg::with_name("tfs-pat")
            .long("tfs-pat")
            .takes_value(true)
            .value_name("ACCESS_TOKEN")
            .help("If this option is set, instead of trying to read an OAuth access token for the \
                the TFS from an environment variable, uses the given personal access token to talk \
                to the TFS REST API. The user this token belongs to must have read access to Work \
                Items!"))
        .get_matches();

    let logger = Logger::new(matches.is_present("verbose"));
    let env_reader = EnvReader::new(|name| std::env::var(name).ok());
    let tfs_access_token = matches.value_of("tfs-pat");
    let app = App::new(&logger, env_reader, tfs_access_token);
    logger.log(&format!(
        "teamscale-timestamp v{} trying to determine branch + timestamp for an external upload",
        version
    ));

    let opt_branch = matches
        .value_of("branch")
        .map(|branch| branch.to_string())
        .or_else(|| app.guess_branch());
    let opt_timestamp = app.guess_timestamp();

    let debug_help = "Run with --verbose for further information. If you believe this is a bug \
        in this program, please run this program with --verbose and send its output plus a detailed \
        bug report to support@teamscale.com";

    match opt_branch {
        None => panic!(
            "Couldn't resolve the branch. Try manually passing the branch with --branch. {}",
            debug_help
        ),
        Some(branch) => match opt_timestamp {
            None => panic!("Couldn't resolve the timestamp. {}", debug_help),
            Some(timestamp) => output(&app, branch, timestamp, matches.value_of("revfile")),
        },
    }
}

fn output(app: &App, branch: String, timestamp: String, opt_revision_txt: Option<&str>) {
    match opt_revision_txt {
        Some(revision_text) => {
            let result = app.write_revision_txt(
                &format!("{}:{}", branch, timestamp),
                Path::new(revision_text),
            );
            match result {
                Err(error) => panic!(
                    "Could not write timestamp to file {}: {}",
                    revision_text,
                    error.to_string()
                ),
                _ => (),
            }
        }
        None => println!("{}:{}", branch, timestamp),
    }
}
