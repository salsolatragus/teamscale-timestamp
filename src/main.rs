extern crate clap;

use clap::Arg;

use crate::app::App;

mod app;
mod git;
mod svn;
mod utils;

fn main() {
    let matches = clap::App::new("teamscale-timestamp")
        .about("Tries to determine the value for the ?t= parameter when uploading external data to Teamscale. \
            Run this command from within the working directory of your version control system checkout.")
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
        .get_matches();

    let app = App::new(matches.is_present("verbose"));

    let opt_branch = matches.value_of("branch").map(|branch| branch.to_string())
        .or_else(|| app.guess_branch());
    let opt_timestamp = app.guess_timestamp();

    match opt_branch {
        None => panic!("Couldn't resolve the branch"),
        Some(branch) => match opt_timestamp {
            None => panic!("Couldn't resolve the timestamp"),
            Some(timestamp) => println!("{}:{}", branch, timestamp),
        },
    }
}
