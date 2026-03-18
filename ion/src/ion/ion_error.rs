use crate::parser::ParserError;
use std::{error, fmt};

/// Errors returned by high-level Ion parsing and access APIs.
#[derive(Clone, Debug)]
pub enum IonError {
    /// A requested section does not exist.
    MissingSection(String),
    /// A requested dictionary field does not exist.
    MissingValue(String),
    /// A generic parse failure without structured parser errors.
    ParseError,
    /// One or more structured parser errors collected while reading the document.
    ParserErrors(Vec<ParserError>),
}

impl error::Error for IonError {
    fn description(&self) -> &'static str {
        "IonError"
    }
}

impl fmt::Display for IonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingSection(section) => write!(f, "missing section: {section}"),
            Self::MissingValue(value) => write!(f, "missing value: {value}"),
            Self::ParseError => write!(f, "parse error"),
            Self::ParserErrors(errors) => {
                if errors.is_empty() {
                    write!(f, "parse errors")
                } else if errors.len() == 1 {
                    write!(f, "{}", errors[0])
                } else {
                    writeln!(f, "{} parse errors:", errors.len())?;
                    for (index, err) in errors.iter().enumerate() {
                        writeln!(f, "{}. {}", index + 1, err)?;
                    }
                    Ok(())
                }
            }
        }
    }
}
