use crate::{Dictionary, FromIon, IonError, Row};
use std::str::FromStr;

/// A typed Ion value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Quoted or unquoted string data.
    String(Box<str>),
    /// Signed integer value.
    Integer(i64),
    /// Floating-point value.
    Float(f64),
    /// Boolean value.
    Boolean(bool),
    /// Array of nested values.
    Array(Row),
    /// Nested dictionary value.
    Dictionary(Dictionary),
}

impl Value {
    /// Creates a string value.
    #[must_use]
    pub fn new_string(value: &str) -> Self {
        Value::String(value.into())
    }

    /// Creates a one-element array containing a string value.
    #[must_use]
    pub fn new_string_array(value: &str) -> Self {
        Self::new_array(Self::new_string(value))
    }

    /// Creates a one-element array containing `value`.
    #[must_use]
    pub fn new_array(value: Value) -> Self {
        Value::Array(vec![value])
    }

    /// Returns the human-readable variant name.
    #[must_use]
    pub fn type_str(&self) -> &'static str {
        match self {
            Value::String(..) => "string",
            Value::Integer(..) => "integer",
            Value::Float(..) => "float",
            Value::Boolean(..) => "boolean",
            Value::Array(..) => "array",
            Value::Dictionary(..) => "dictionary",
        }
    }

    /// Returns the inner string slice when this is [`Value::String`].
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    /// Returns `true` when this is [`Value::String`].
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Alias for [`as_string`](Self::as_string).
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.as_string()
    }

    /// Returns the integer value when this is [`Value::Integer`].
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the float value when this is [`Value::Float`].
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the boolean value when this is [`Value::Boolean`].
    #[must_use]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the array contents when this is [`Value::Array`].
    #[must_use]
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the dictionary contents when this is [`Value::Dictionary`].
    #[must_use]
    pub fn as_dictionary(&self) -> Option<&Dictionary> {
        match self {
            Value::Dictionary(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a nested dictionary field by name.
    ///
    /// Returns `None` when this is not [`Value::Dictionary`] or the key is missing.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Dictionary(v) => v.get(name),
            _ => None,
        }
    }

    /// # Errors
    ///
    /// Returns any error produced by `F::from_ion`.
    pub fn from_ion<F>(&self) -> Result<F, F::Err>
    where
        F: FromIon<Value>,
    {
        F::from_ion(self)
    }

    /// # Errors
    ///
    /// Returns any parse error produced by `F`.
    pub fn parse<F>(&self) -> Result<F, F::Err>
    where
        F: FromStr,
    {
        match self.as_string() {
            Some(s) => s.parse(),
            None => self.to_string().parse(),
        }
    }
}

impl FromStr for Value {
    type Err = IonError;

    fn from_str(s: &str) -> Result<Value, IonError> {
        Ok(Value::String(s.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Dictionary, Value};
    use pretty_assertions::assert_eq;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct ParseTestCase {
        raw: &'static str,
        expected_integer: Option<i64>,
        expected_float: Option<f64>,
    }

    #[derive(Debug)]
    struct HelperTestCase {
        value: Value,
        expected_type: &'static str,
        is_string: bool,
        expected_string: Option<&'static str>,
        expected_float: Option<f64>,
        expected_array_len: Option<usize>,
        expected_dictionary_lookup: Option<&'static str>,
    }

    const INTEGER_CASE: ParseTestCase = ParseTestCase {
        raw: "1",
        expected_integer: Some(1),
        expected_float: None,
    };
    const FLOAT_CASE: ParseTestCase = ParseTestCase {
        raw: "4.0",
        expected_integer: None,
        expected_float: Some(4.0),
    };
    #[test_case(&INTEGER_CASE; "integer")]
    fn integer(case: &ParseTestCase) {
        let value: Value = case.raw.parse().unwrap();
        let actual: i64 = value.parse().unwrap();
        assert_eq!(case.expected_integer.unwrap(), actual);
    }

    #[test_case(&FLOAT_CASE; "float")]
    fn float(case: &ParseTestCase) {
        let value: Value = case.raw.parse().unwrap();
        let parsed: f64 = value.parse().unwrap();
        assert!((case.expected_float.unwrap() - parsed).abs() < f64::EPSILON);
    }

    static STRING_HELPER_CASE: LazyLock<HelperTestCase> = LazyLock::new(|| HelperTestCase {
        value: Value::new_string("foo"),
        expected_type: "string",
        is_string: true,
        expected_string: Some("foo"),
        expected_float: None,
        expected_array_len: None,
        expected_dictionary_lookup: None,
    });
    static ARRAY_HELPER_CASE: LazyLock<HelperTestCase> = LazyLock::new(|| HelperTestCase {
        value: Value::new_string_array("foo"),
        expected_type: "array",
        is_string: false,
        expected_string: None,
        expected_float: None,
        expected_array_len: Some(1),
        expected_dictionary_lookup: None,
    });
    static FLOAT_HELPER_CASE: LazyLock<HelperTestCase> = LazyLock::new(|| HelperTestCase {
        value: Value::Float(4.25),
        expected_type: "float",
        is_string: false,
        expected_string: None,
        expected_float: Some(4.25),
        expected_array_len: None,
        expected_dictionary_lookup: None,
    });
    static DICTIONARY_HELPER_CASE: LazyLock<HelperTestCase> = LazyLock::new(|| {
        let dictionary = Dictionary::from([("name".to_owned(), Value::new_string("foo"))]);
        HelperTestCase {
            value: Value::Dictionary(dictionary),
            expected_type: "dictionary",
            is_string: false,
            expected_string: None,
            expected_float: None,
            expected_array_len: None,
            expected_dictionary_lookup: Some("foo"),
        }
    });

    #[test_case(&*STRING_HELPER_CASE; "string helpers")]
    #[test_case(&*ARRAY_HELPER_CASE; "array helpers")]
    #[test_case(&*FLOAT_HELPER_CASE; "float helpers")]
    #[test_case(&*DICTIONARY_HELPER_CASE; "dictionary helpers")]
    fn helpers(case: &HelperTestCase) {
        assert_eq!(case.expected_type, case.value.type_str());
        assert_eq!(case.is_string, case.value.is_string());
        assert_eq!(case.expected_string, case.value.as_str());

        match (case.expected_float, case.value.as_float()) {
            (Some(expected), Some(actual)) => assert!((expected - actual).abs() < f64::EPSILON),
            (None, None) => {}
            _ => panic!("unexpected float result"),
        }

        assert_eq!(case.expected_array_len, case.value.as_array().map(Vec::len));
        assert_eq!(
            case.expected_dictionary_lookup,
            case.value
                .as_dictionary()
                .and_then(|dictionary: &Dictionary| dictionary.get("name"))
                .and_then(Value::as_str)
        );
        assert_eq!(
            case.expected_dictionary_lookup,
            case.value.get("name").and_then(Value::as_str)
        );
    }

    #[test]
    fn parse_uses_display_for_non_string_values() {
        let value = Value::Integer(42);
        let parsed: i64 = value.parse().unwrap();
        assert_eq!(42, parsed);
    }
}
