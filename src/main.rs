use crate::app::App;

mod app;
mod git;
mod svn;
mod utils;

fn main() {
    let app = App::new(true);

    let opt_branch = app.guess_branch();
    let opt_timestamp = app.guess_timestamp();

    match opt_branch {
        None => panic!("Couldn't resolve the branch"),
        Some(branch) => match opt_timestamp {
            None => panic!("Couldn't resolve the timestamp"),
            Some(timestamp) => println!("{}:{}", branch, timestamp),
        },
    }
}
