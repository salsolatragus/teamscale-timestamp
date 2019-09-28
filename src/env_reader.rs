pub trait EnvReader {
    fn env_variable(&self, name: &str) -> Option<String>;
}
