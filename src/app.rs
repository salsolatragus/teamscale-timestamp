use std::path::Path;
use std::option::Option;
use std::string::String;

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

    pub fn guess_branch(&self, directory: &Path) -> Option<String> {
        return None;
    }

    pub fn guess_timestamp(&self, directory: &Path) -> Option<String> {
        return None;
    }
}