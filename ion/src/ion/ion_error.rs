use crate::parser::ParserError;
use std::{error, fmt};

#[derive(Clone, Debug)]
pub enum IonError {
    MissingSection(String),
    MissingValue(String),
    ParseError,
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
