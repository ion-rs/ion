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
    use crate::ion::{FromRow, Value};

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

    impl FromRow for Foo {
        type Err = &'static str;

        fn from_str_iter<'a, I: Iterator<Item = &'a Value>>(mut row: I) -> Result<Self, Self::Err> {
            Ok(Foo {
                foo: parse_next!(row, "foo"),
                bar: parse_next!(row, "bar"),
            })
        }
    }

    #[test]
    fn from_row() {
        let row: Vec<_> = "1|foo"
            .split('|')
            .map(|s| Value::String(s.to_owned()))
            .collect();

        let foo = Foo::from_str_iter(row.iter()).unwrap();

        assert_eq!(
            Foo {
                foo: 1,
                bar: "foo".to_owned(),
            },
            foo
        );
    }
}
