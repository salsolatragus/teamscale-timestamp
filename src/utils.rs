pub trait PeekOption<T> {
    fn if_some<F: FnOnce(&T) -> ()>(self, peeker: F) -> Option<T>;
    fn if_none<F: FnOnce() -> ()>(self, peeker: F) -> Option<T>;
    fn peek_or_default<F: FnOnce(&T) -> ()>(self, peeker: F, default: T) -> Option<T>;
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
