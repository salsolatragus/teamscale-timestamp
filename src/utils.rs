use std::process::Output;

pub trait StrFromOutput<E> {
    fn map_to_stdout(self) -> Result<String, E>;
}

impl<E> StrFromOutput<E> for Result<Output, E> {
    fn map_to_stdout(self) -> Result<String, E> {
        return self.map(|output| std::str::from_utf8(output.stdout.as_ref()).unwrap().to_string());
    }
}

pub trait PeekOption<T> {
    fn if_some<F : FnOnce(&T) -> ()>(self, peeker: F) -> Option<T>;
    fn if_none<F : FnOnce() -> ()>(self, peeker: F) -> Option<T>;
    fn peek_or_default<F : FnOnce(&T) -> ()>(self, peeker: F, default: T) -> Option<T>;
}

impl<T> PeekOption<T> for Option<T> {
    fn if_some<F: FnOnce(&T) -> ()>(self, peeker: F) -> Option<T> {
        match self {
            Some(ref value) => peeker(value),
            _ => (),
        }
        return self;
    }
    fn if_none<F: FnOnce() -> ()>(self, peeker: F) -> Option<T> {
        match self {
            None => peeker(),
            _ => (),
        }
        return self;
    }

    fn peek_or_default<F: FnOnce(&T) -> ()>(self, peeker: F, default: T) -> Option<T> {
        match self {
            Some(ref value) => peeker(value),
            None => peeker(&default),
        }
        return self;
    }
}
