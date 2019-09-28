pub trait Logger {
    fn log<S>(&self, message: S)
    where
        S: Into<String>;
}
