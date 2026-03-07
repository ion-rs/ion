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
    use std::str::FromStr;

    #[test]
    fn string() {
        let v = Value::String("foo".to_owned());
        let s = String::from_ion(&v).unwrap();
        assert_eq!("foo", s);

        let s: String = v.from_ion().unwrap();
        assert_eq!("foo", s);
    }

    #[test]
    fn option_string() {
        let v = Value::from_str("foo").unwrap();
        let os: Option<String> = v.from_ion().unwrap();
        assert_eq!(Some("foo".to_owned()), os);

        let v = Value::from_str("").unwrap();
        let os: Option<String> = v.from_ion().unwrap();
        assert_eq!(None, os);
    }

    #[test]
    fn u32() {
        let v = Value::from_str("16").unwrap();
        let u: u32 = v.from_ion().unwrap();
        assert_eq!(16, u);
    }

    #[test]
    fn bool() {
        let v = Value::from_str("true").unwrap();
        let u: bool = v.from_ion().unwrap();
        assert!(u);

        let v = Value::from_str("false").unwrap();
        let u: bool = v.from_ion().unwrap();
        assert!(!u);

        let v = Value::from_str("").unwrap();
        let u: Result<bool, _> = v.from_ion();
        assert!(u.is_err());
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

    #[test]
    fn from_ion_section() {
        let section = Section::new();
        let foo: Foo = section.parse().unwrap();
        assert_eq!(1, foo.a);
        assert_eq!("foo", foo.b);
    }
}
