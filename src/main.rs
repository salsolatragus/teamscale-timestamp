use std::path::Path;
use crate::app::App;

mod app;
mod git;
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

/*#[derive(Debug)]
enum TsError {
    SvnUrlParsingFailed(url: &str),
}

impl Error for TsError {}

fn find_branch_in_svn_url(url: &str) -> Result<&str, ParseError> {
    let regex = Regex::new("(?:branches|tags)/(?P<branch>[^/]+)|(?P<trunk>trunk)").unwrap();
    let captures = regex.captures(url)?;
    match captures.name("branch") {
        Some(m) => return Ok(m.as_str()),
        _ => (),
    }

    return match captures.name("trunk") {
        Some(_) => Ok("trunk"),
        _ => Err(ParseError::(format!("could not parse url {}", url))),
    };
}

fn svn_branch() -> Result<String, dyn Error> {
    let environment_result = std::env::var("SVN_URL");
    match environment_result {
        Ok(url) => return find_branch_in_svn_url(&url),
        _ => (),
    }

    return Command::new("svn")
        .args(&["info", "--show-item", "url"])
        .output()
        .and_then(|output| find_branch_in_svn_url(str::from_utf8(output.stdout)));
}

fn branch() -> Result<String, VarError> {
    return std::env::var("GIT_BRANCH")
        .or(std::env::var("BRANCH"))
        .or(svn_branch());
}

fn parse_svn_timestamp(date_string: &str) -> Result<&str, dyn Error> {
    let date = DateTime::parse_from_rfc3339(date_string)?;
    return format!("{}000", date.timestamp());
}

fn timestamp() -> Result<String, dyn Error> {
    let git_timestamp_result = Command::new("git")
        .args(&["--no-pager", "log", "-n1", "--format=\"%ct000\""])
        .output()
        .map(|output| str::from_utf8(output.stdout));

    return git_timestamp_result.or_else(|| {
        let svn_date_result = Command::new("svn")
            .args(&["info", "--show-item", "last-changed-date"])
            .env("LANG", "C")
            .output()
            .map(|output| parse_svn_timestamp(str::from_utf8(output.stdout)));
        svn_date_result;
    });
}*/

