use std::process::Command;

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

pub fn run(program: &str, args: &[&str], configurator: fn(&mut Command) -> &mut Command) -> Result<String, String> {
    let opt_output = configurator(Command::new(program).args(args)).output();

    match opt_output {
        Ok(output) => {
            if !output.status.success() {
                return Err(format!("{} {} failed with exit code {}", program, args.join(" "),
                                   output.status.code().unwrap_or(-999)));
            }
            return Ok(std::str::from_utf8(output.stdout.as_ref()).unwrap().to_string());
        }
        Err(error) => {
            return Err(format!("{} {} failed: {}", program, args.join(" "), error.to_string()));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils::run;

    ///#[test]
    fn test_run() {
        if cfg!(windows) {
            let output = run("cmd", &["/c", "echo Windows"], |command| command);
            assert!(output.is_ok(), "output was not ok: {:?}", output);
            assert!(output.clone().unwrap().contains("Windows"),
                    "output did not contain expected text: {:?}", output);
        }
        if cfg!(unix) {
            let output = run("cat", &["--help"], |command| command);
            assert!(output.is_ok(), "output was not ok: {:?}", output);
            assert!(output.clone().unwrap().contains("Usage:"),
                    "output did not contain expected text: {:?}", output);

            assert!(run("false", &[], |command| command).is_err());
        }

        assert!(run("does-not-exist", &["--nonsense"], |command| command).is_err());
    }
}
