use std::boxed::Box;

pub struct EnvReader<'a> {
    reader: Box<dyn Fn(&str) -> Option<String> + 'a>,
}

impl<'a> EnvReader<'a> {
    pub fn new(reader: impl Fn(&str) -> Option<String> + 'a) -> EnvReader<'a> {
        return EnvReader {
            reader: Box::new(reader),
        };
    }

    pub fn env_variable(&self, name: &str) -> Option<String> {
        return (self.reader)(name);
    }
}
