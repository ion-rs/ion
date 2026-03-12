use crate::Row;
use crate::ion::Value;

pub trait FromRow
where
    Self: Sized,
{
    type Err;

    /// # Errors
    ///
    /// Returns an error when the row values cannot be converted into `Self`.
    fn from_str_iter<'a, I>(row: I) -> Result<Self, Self::Err>
    where
        I: Iterator<Item = &'a Value>;
}

pub trait ParseRow
where
    Self: Sized,
{
    type Err;

    /// # Errors
    ///
    /// Returns an error when parsing the row into `F` fails.
    fn parse<F: FromRow>(&self) -> Result<F, F::Err>;
}

impl ParseRow for Row {
    type Err = ();

    fn parse<F: FromRow>(&self) -> Result<F, F::Err> {
        F::from_str_iter(self.iter())
    }
}

#[cfg(test)]
mod tests {
    use crate::ion::{FromRow, ParseRow, Value};
    use pretty_assertions::assert_eq;
    use std::sync::LazyLock;
    use test_case::test_case;

    macro_rules! parse_next {
        ($row:expr, $err:expr) => {{
            match $row.next() {
                Some(v) => match v.parse() {
                    Ok(v) => v,
                    Err(_) => return Err($err),
                },
                None => return Err($err),
            }
        }};
    }

    #[derive(Debug, PartialEq)]
    struct Foo {
        foo: u32,
        bar: String,
    }

    #[derive(Debug)]
    struct TestCase {
        row: Vec<Value>,
        expected: Foo,
    }

    #[derive(Debug)]
    struct ErrorTestCase {
        row: Vec<Value>,
        expected_error: &'static str,
    }

    impl FromRow for Foo {
        type Err = &'static str;

        fn from_str_iter<'a, I: Iterator<Item = &'a Value>>(mut row: I) -> Result<Self, Self::Err> {
            Ok(Foo {
                foo: parse_next!(row, "foo"),
                bar: parse_next!(row, "bar"),
            })
        }
    }

    static FROM_ROW_CASE: LazyLock<TestCase> = LazyLock::new(|| TestCase {
        row: "1|foo"
            .split('|')
            .map(|s| Value::String(s.to_owned()))
            .collect(),
        expected: Foo {
            foo: 1,
            bar: "foo".to_owned(),
        },
    });
    static PARSE_ROW_CASE: LazyLock<TestCase> = LazyLock::new(|| TestCase {
        row: "2|bar"
            .split('|')
            .map(|s| Value::String(s.to_owned()))
            .collect(),
        expected: Foo {
            foo: 2,
            bar: "bar".to_owned(),
        },
    });
    static FROM_ROW_ERROR_CASE: LazyLock<ErrorTestCase> = LazyLock::new(|| ErrorTestCase {
        row: vec![Value::String("oops".to_owned())],
        expected_error: "foo",
    });

    #[test_case(&*FROM_ROW_CASE; "parses row")]
    fn from_row(case: &TestCase) {
        let actual = Foo::from_str_iter(case.row.iter()).unwrap();
        assert_eq!(case.expected, actual);
    }

    #[test_case(&*PARSE_ROW_CASE; "parse row trait")]
    fn parse_row(case: &TestCase) {
        let actual: Foo = case.row.parse().unwrap();
        assert_eq!(case.expected, actual);
    }

    #[test_case(&*FROM_ROW_ERROR_CASE; "from row error")]
    fn from_row_error(case: &ErrorTestCase) {
        let actual = Foo::from_str_iter(case.row.iter());
        assert_eq!(Err(case.expected_error), actual);
    }
}
