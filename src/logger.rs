pub struct Logger {
    verbose: bool,
}

impl Logger {
    pub fn new(verbose: bool) -> Logger {
        Logger { verbose }
    }

    pub fn log<S>(&self, message: S)
    where
        S: Into<String>,
    {
        if self.verbose {
            println!("{}", message.into());
        }
    }
}
