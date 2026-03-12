mod display;
mod from_ion;
mod from_row;
mod ion_error;
mod section;
mod value;

pub use self::from_ion::*;
pub use self::from_row::*;
pub use self::ion_error::*;
pub use self::section::*;
pub use self::value::*;
use crate::Parser;
use std::collections::BTreeMap;
use std::str;

#[derive(Clone, Debug)]
pub struct Ion {
    sections: BTreeMap<String, Section>,
}

impl Ion {
    #[must_use]
    pub fn new(sections: BTreeMap<String, Section>) -> Ion {
        Ion { sections }
    }

    /// # Errors
    ///
    /// Returns a parser error when the input cannot be parsed into a valid Ion document.
    pub fn from_str_filtered(s: &str, accepted_sections: Vec<&str>) -> Result<Self, IonError> {
        parser_to_ion(Parser::new_filtered(s, accepted_sections))
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Section> {
        self.sections.get(key)
    }

    /// Returns a mutable reference to the section associated with the given key.
    ///
    /// If a section exists for the provided key, a mutable reference to that section is returned.
    /// If no section is associated with the key, `None` is returned.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Section> {
        self.sections.get_mut(key)
    }

    /// Retrieves a key-value pair from the sections.
    ///
    /// This method attempts to find a section by its key within the collection of sections.
    /// If the section exists, it returns an `Option` containing a tuple of the key as a
    /// reference to a `String` and the value as a reference to a `Section`. If the key
    /// does not exist within the sections, it returns `None`.
    ///
    /// # Returns
    ///
    /// Returns `Option<(&String, &Section)>`. If the key is found, the return value is
    /// `Some((&String, &Section))`, where the first element is a reference to the key
    /// and the second element is a reference to the corresponding `Section`. If the key
    /// is not found, it returns `None`.
    #[must_use]
    pub fn get_key_value(&self, key: &str) -> Option<(&String, &Section)> {
        self.sections.get_key_value(key)
    }

    /// # Errors
    ///
    /// Returns [`IonError::MissingSection`] when the key does not exist.
    pub fn fetch(&self, key: &str) -> Result<&Section, IonError> {
        self.get(key)
            .ok_or_else(|| IonError::MissingSection(key.to_owned()))
    }

    pub fn remove(&mut self, key: &str) -> Option<Section> {
        self.sections.remove(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Section)> {
        self.sections.iter()
    }
}

impl str::FromStr for Ion {
    type Err = IonError;

    fn from_str(s: &str) -> Result<Ion, IonError> {
        parser_to_ion(Parser::new(s))
    }
}

fn parser_to_ion(mut parser: Parser) -> Result<Ion, IonError> {
    match parser.read() {
        Some(ion) => Ok(Ion::new(ion)),
        None => Err(IonError::ParserErrors(parser.errors)),
    }
}

#[macro_export]
macro_rules! ion {
    ($raw:expr) => {{ $raw.parse::<Ion>().expect("Failed parsing to 'Ion'") }};
}

#[macro_export]
macro_rules! ion_filtered {
    ($raw:expr, $accepted_sections:expr) => {
        Ion::from_str_filtered($raw, $accepted_sections)
            .expect("Failed parsing by 'from_str_filtered' to 'Ion'")
    };
}

#[cfg(test)]
mod tests {
    use crate::{Ion, Value};
    use pretty_assertions::assert_eq;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct ValueConversionTestCase {
        value: Value,
        expected_string: Option<&'static str>,
        expected_boolean: Option<bool>,
        expected_integer: Option<i64>,
        expected_str: Option<&'static str>,
    }

    #[derive(Debug)]
    struct RowCountTestCase {
        raw: &'static str,
        accepted_sections: Option<Vec<&'static str>>,
        section: &'static str,
        expected_rows: usize,
        expected_missing_section: Option<&'static str>,
    }

    static STRING_VALUE_CASE: LazyLock<ValueConversionTestCase> =
        LazyLock::new(|| ValueConversionTestCase {
            value: Value::String("foo".into()),
            expected_string: Some("foo"),
            expected_boolean: None,
            expected_integer: None,
            expected_str: Some("foo"),
        });
    static BOOLEAN_VALUE_CASE: LazyLock<ValueConversionTestCase> =
        LazyLock::new(|| ValueConversionTestCase {
            value: Value::Boolean(true),
            expected_string: None,
            expected_boolean: Some(true),
            expected_integer: None,
            expected_str: None,
        });
    static INTEGER_VALUE_CASE: LazyLock<ValueConversionTestCase> =
        LazyLock::new(|| ValueConversionTestCase {
            value: Value::Integer(1),
            expected_string: None,
            expected_boolean: None,
            expected_integer: Some(1),
            expected_str: None,
        });

    static ROWS_WITHOUT_HEADER_CASE: LazyLock<RowCountTestCase> =
        LazyLock::new(|| RowCountTestCase {
            raw: r"
            [FOO]
            |1||2|
            |1|   |2|
            |1|2|3|
        ",
            accepted_sections: None,
            section: "FOO",
            expected_rows: 3,
            expected_missing_section: None,
        });
    static ROWS_WITH_HEADER_CASE: LazyLock<RowCountTestCase> = LazyLock::new(|| RowCountTestCase {
        raw: r"
            [FOO]
            | 1 | 2 | 3 |
            |---|---|---|
            |1||2|
            |1|   |2|
        ",
        accepted_sections: None,
        section: "FOO",
        expected_rows: 2,
        expected_missing_section: None,
    });
    static NO_ROWS_WITH_HEADER_CASE: LazyLock<RowCountTestCase> =
        LazyLock::new(|| RowCountTestCase {
            raw: r"
            [FOO]
            | 1 | 2 | 3 |
            |---|---|---|
        ",
            accepted_sections: None,
            section: "FOO",
            expected_rows: 0,
            expected_missing_section: None,
        });
    static FILTERED_SECTION_CASE: LazyLock<RowCountTestCase> = LazyLock::new(|| RowCountTestCase {
        raw: r"
            [FOO]
            |1||2|
            |1|   |2|
            |1|2|3|
            [BAR]
            |1||2|
        ",
        accepted_sections: Some(vec!["FOO"]),
        section: "FOO",
        expected_rows: 3,
        expected_missing_section: Some("BAR"),
    });

    #[test_case(&*STRING_VALUE_CASE; "string")]
    #[test_case(&*BOOLEAN_VALUE_CASE; "boolean")]
    #[test_case(&*INTEGER_VALUE_CASE; "integer")]
    fn value_accessors(case: &ValueConversionTestCase) {
        assert_eq!(
            case.expected_string.map(str::to_owned).as_ref(),
            case.value.as_string()
        );
        assert_eq!(case.expected_boolean, case.value.as_boolean());
        assert_eq!(case.expected_integer, case.value.as_integer());
        assert_eq!(case.expected_str, case.value.as_str());
    }

    #[test_case(&*ROWS_WITHOUT_HEADER_CASE; "without header")]
    #[test_case(&*ROWS_WITH_HEADER_CASE; "with header")]
    #[test_case(&*NO_ROWS_WITH_HEADER_CASE; "header only")]
    #[test_case(&*FILTERED_SECTION_CASE; "filtered section")]
    fn rows_without_header(case: &RowCountTestCase) {
        let ion = match &case.accepted_sections {
            Some(accepted_sections) => {
                Ion::from_str_filtered(case.raw, accepted_sections.clone()).unwrap()
            }
            None => case.raw.parse::<Ion>().unwrap(),
        };

        let rows = ion.get(case.section).unwrap().rows_without_header();
        assert_eq!(case.expected_rows, rows.len());

        if let Some(section) = case.expected_missing_section {
            assert_eq!(None, ion.get(section));
        }
    }
}
