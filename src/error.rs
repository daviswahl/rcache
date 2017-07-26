use std::error;
use std::fmt;
use std::io;

/// `ErrorKind`
#[derive(Debug)]
pub enum ErrorKind {
    InvalidData,
    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Other => "Other"
        };
        write!(f, "{}", s)
    }
}

/// `Error`
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: String,
}

impl Error {
    pub fn new(kind: ErrorKind, description: &str) -> Self {
        Self{kind: kind, description: description.to_owned()}
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.description.as_str()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.description)
    }
}

impl From<&'static str> for Error {
    fn from(e: &'static str) -> Self {
        Error::new(ErrorKind::Other, e)
    }
}
impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

