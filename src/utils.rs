use std::process::Output;

pub trait StrFromOutput<E> {
    fn map_to_stdout(self) -> Result<String, E>;
}

impl<E> StrFromOutput<E> for Result<Output, E> {
    fn map_to_stdout(self) -> Result<String, E> {
        return self.map(|output| std::str::from_utf8(output.stdout.as_ref()).unwrap().to_string());
    }
}