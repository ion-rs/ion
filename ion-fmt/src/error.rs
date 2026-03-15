use std::error::Error;
use std::fmt;

/// Error returned by file-based formatting operations.
#[derive(Debug)]
pub enum FormatError {
    /// Filesystem read/write error.
    Io(std::io::Error),
    /// Ion parse error.
    Parse(ion::IonError),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Parse(error) => write!(f, "{error}"),
        }
    }
}

impl Error for FormatError {}

impl From<std::io::Error> for FormatError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ion::IonError> for FormatError {
    fn from(value: ion::IonError) -> Self {
        Self::Parse(value)
    }
}
