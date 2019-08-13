use std::path::Path;
use crate::app::App;
use regex::Regex;

mod app;
mod git;
mod svn;
mod utils;

fn main() {
    let app = App::new(true);
    let directory = Path::new(".");

    let opt_branch = app.guess_branch(directory);
    let opt_timestamp = app.guess_timestamp(directory);

    match opt_branch {
        None => panic!("Couldn't resolve the branch"),
        Some(branch) => match opt_timestamp {
            None => panic!("Couldn't resolve the timestamp"),
            Some(timestamp) => println!("{}:{}", branch, timestamp),
        },
    }
}

/*

    let environment_result = std::env::var("SVN_URL");
    match environment_result {
        Ok(url) => return find_branch_in_svn_url(&url),
        _ => (),
    }


fn branch() -> Result<String, VarError> {
    return std::env::var("GIT_BRANCH")
        .or(std::env::var("BRANCH"))
        .or(svn_branch());
}
*/

