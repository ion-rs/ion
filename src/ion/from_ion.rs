use crate::ion::Value;
use std::num::ParseIntError;
use std::str::ParseBoolError;

pub trait FromIon<T>
where
    Self: Sized,
{
    type Err;

    /// # Errors
    ///
    /// Returns an error when `T` cannot be converted into `Self`.
    fn from_ion(_: &T) -> Result<Self, Self::Err>;
}

impl FromIon<Value> for String {
    type Err = ();

    fn from_ion(value: &Value) -> Result<Self, Self::Err> {
        value
            .as_string()
            .map(std::borrow::ToOwned::to_owned)
            .ok_or(())
    }
}

impl FromIon<Value> for Option<String> {
    type Err = ();

    fn from_ion(value: &Value) -> Result<Self, Self::Err> {
        value
            .as_string()
            .map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_owned())
                }
            })
            .ok_or(())
    }
}

macro_rules! from_ion_value_int_impl {
     ($($t:ty)*) => {$(
         impl FromIon<Value> for $t {
             type Err = ParseIntError;

             fn from_ion(value: &Value) -> Result<Self, Self::Err> {
                match value.as_string() {
                    Some(s) => Ok(s.parse()?),
                    None => "".parse()
                }
             }
         }
     )*}
 }

from_ion_value_int_impl! { isize i8 i16 i32 i64 usize u8 u16 u32 u64 }

impl FromIon<Value> for bool {
    type Err = ParseBoolError;

    fn from_ion(value: &Value) -> Result<Self, Self::Err> {
        match value.as_string() {
            Some(s) => Ok(s.parse()?),
            None => "".parse(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ion::{FromIon, Section, Value};
    use pretty_assertions::assert_eq;
    use std::str::FromStr;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct StringTestCase {
        value: Value,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct OptionStringTestCase {
        value: Value,
        expected: Option<&'static str>,
    }

    #[derive(Debug)]
    struct U32TestCase {
        value: Value,
        expected: u32,
    }

    #[derive(Debug)]
    struct BoolTestCase {
        value: Value,
        expected: Result<bool, ()>,
    }

    #[derive(Debug)]
    struct SectionTestCase {
        expected_a: u32,
        expected_b: &'static str,
    }

    static STRING_CASE: LazyLock<StringTestCase> = LazyLock::new(|| StringTestCase {
        value: Value::String("foo".to_owned()),
        expected: "foo",
    });
    static OPTION_STRING_SOME_CASE: LazyLock<OptionStringTestCase> =
        LazyLock::new(|| OptionStringTestCase {
            value: Value::from_str("foo").unwrap(),
            expected: Some("foo"),
        });
    static OPTION_STRING_NONE_CASE: LazyLock<OptionStringTestCase> =
        LazyLock::new(|| OptionStringTestCase {
            value: Value::from_str("").unwrap(),
            expected: None,
        });
    static U32_CASE: LazyLock<U32TestCase> = LazyLock::new(|| U32TestCase {
        value: Value::from_str("16").unwrap(),
        expected: 16,
    });
    static BOOL_TRUE_CASE: LazyLock<BoolTestCase> = LazyLock::new(|| BoolTestCase {
        value: Value::from_str("true").unwrap(),
        expected: Ok(true),
    });
    static BOOL_FALSE_CASE: LazyLock<BoolTestCase> = LazyLock::new(|| BoolTestCase {
        value: Value::from_str("false").unwrap(),
        expected: Ok(false),
    });
    static BOOL_ERROR_CASE: LazyLock<BoolTestCase> = LazyLock::new(|| BoolTestCase {
        value: Value::from_str("").unwrap(),
        expected: Err(()),
    });
    static SECTION_CASE: LazyLock<SectionTestCase> = LazyLock::new(|| SectionTestCase {
        expected_a: 1,
        expected_b: "foo",
    });

    #[test_case(&*STRING_CASE; "string")]
    fn string(case: &StringTestCase) {
        let actual = String::from_ion(&case.value).unwrap();
        assert_eq!(case.expected, actual);

        let actual: String = case.value.from_ion().unwrap();
        assert_eq!(case.expected, actual);
    }

    #[test_case(&*OPTION_STRING_SOME_CASE; "some")]
    #[test_case(&*OPTION_STRING_NONE_CASE; "none")]
    fn option_string(case: &OptionStringTestCase) {
        let actual: Option<String> = case.value.from_ion().unwrap();
        assert_eq!(case.expected.map(str::to_owned), actual);
    }

    #[test_case(&*U32_CASE; "u32")]
    fn u32(case: &U32TestCase) {
        let actual: u32 = case.value.from_ion().unwrap();
        assert_eq!(case.expected, actual);
    }

    #[test_case(&*BOOL_TRUE_CASE; "bool_true")]
    #[test_case(&*BOOL_FALSE_CASE; "bool_false")]
    #[test_case(&*BOOL_ERROR_CASE; "error")]
    fn bool(case: &BoolTestCase) {
        let actual: Result<bool, _> = case.value.from_ion();
        assert_eq!(case.expected, actual.map_err(|_| ()));
    }

    struct Foo {
        a: u32,
        b: String,
    }

    impl FromIon<Section> for Foo {
        type Err = ();

        fn from_ion(_section: &Section) -> Result<Self, Self::Err> {
            Ok(Foo {
                a: 1,
                b: "foo".to_owned(),
            })
        }
    }

    #[test_case(&*SECTION_CASE; "section")]
    fn from_ion_section(case: &SectionTestCase) {
        let section = Section::new();
        let foo: Foo = section.parse().unwrap();
        assert_eq!(case.expected_a, foo.a);
        assert_eq!(case.expected_b, foo.b);
    }
}
