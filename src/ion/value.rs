use crate::{Dictionary, FromIon, IonError, Row};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Row),
    Dictionary(Dictionary),
}

impl Value {
    #[must_use]
    pub fn new_string(value: &str) -> Self {
        Value::String(value.to_owned())
    }

    #[must_use]
    pub fn new_string_array(value: &str) -> Self {
        Self::new_array(Self::new_string(value))
    }

    #[must_use]
    pub fn new_array(value: Value) -> Self {
        Value::Array(vec![value])
    }

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

    #[must_use]
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_dictionary(&self) -> Option<&Dictionary> {
        match self {
            Value::Dictionary(v) => Some(v),
            _ => None,
        }
    }

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
        Ok(Value::String(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use crate::Value;
    use pretty_assertions::assert_eq;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct TestCase {
        raw: &'static str,
        expected_integer: Option<i64>,
        expected_float: Option<f64>,
    }

    static INTEGER_CASE: LazyLock<TestCase> = LazyLock::new(|| TestCase {
        raw: "1",
        expected_integer: Some(1),
        expected_float: None,
    });
    static FLOAT_CASE: LazyLock<TestCase> = LazyLock::new(|| TestCase {
        raw: "4.0",
        expected_integer: None,
        expected_float: Some(4.0),
    });

    #[test_case(&*INTEGER_CASE; "integer")]
    fn integer(case: &TestCase) {
        let value: Value = case.raw.parse().unwrap();
        assert_eq!(case.expected_integer.unwrap(), value.parse().unwrap());
    }

    #[test_case(&*FLOAT_CASE; "float")]
    fn float(case: &TestCase) {
        let value: Value = case.raw.parse().unwrap();
        let parsed: f64 = value.parse().unwrap();
        assert!((case.expected_float.unwrap() - parsed).abs() < f64::EPSILON);
    }
}
