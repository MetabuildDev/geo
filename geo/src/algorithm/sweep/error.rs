use std::fmt;

#[derive(Clone, Debug)]
pub enum Error {
    Unhandled(&'static str),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Unhandled(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}
