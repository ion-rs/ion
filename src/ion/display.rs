use crate::{Ion, Section, Value};
use std::fmt::{self, Write};

impl fmt::Display for Ion {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (name, section) in &self.sections {
            f.write_fmt(format_args!("[{name}]\n"))?;
            section.fmt(f)?;
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (k, v) in &self.dictionary {
            f.write_fmt(format_args!("{k} = {v:#}\n"))?;
        }

        for row in &self.rows {
            for cell in row {
                fmt::Display::fmt(&format!("| {cell} "), f)?;
            }
            f.write_str("|\n")?;
        }

        Ok(())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Value::String(v) => {
                if f.alternate() {
                    f.write_char('"')?;
                    for c in v.chars() {
                        match c {
                            '\\' => f.write_str("\\\\")?,
                            '\n' => f.write_str("\\n")?,
                            '\"' => f.write_str("\\\"")?,
                            _ => f.write_char(c)?,
                        }
                    }
                    f.write_char('"')?;
                } else {
                    let mut escaping = false;
                    for c in v.chars() {
                        match (escaping, c) {
                            (false, '\\') => {
                                escaping = true;
                                f.write_char('\\')?;
                                continue;
                            }
                            (false, '\n') | (true, 'n') => f.write_str("\\n")?,
                            (false, '\t') | (true, 't') => f.write_str("\\t")?,
                            (false | true, '|') => f.write_str("\\|")?,

                            (true, '\\') => f.write_char('\\')?,

                            (_, c) => f.write_char(c)?,
                        }
                        escaping = false;
                    }
                }
                Ok(())
            }

            Value::Integer(v) => v.fmt(f),
            Value::Float(v) => v.fmt(f),
            Value::Boolean(v) => v.fmt(f),

            Value::Array(v) => {
                f.write_str("[ ")?;

                let mut first = true;

                for i in v {
                    if first {
                        first = false;
                    } else {
                        f.write_str(", ")?;
                    }

                    write!(f, "{i:#}")?;
                }

                f.write_str(" ]")
            }

            Value::Dictionary(d) => {
                f.write_str("{ ")?;

                let mut first = true;

                for (k, v) in d {
                    if first {
                        first = false;
                    } else {
                        f.write_str(", ")?;
                    }

                    k.fmt(f)?;
                    f.write_str(" = ")?;

                    write!(f, "{v:#}")?;
                }

                f.write_str(" }")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct ValueDisplayTestCase {
        value: Value,
        alternate: bool,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct SectionDisplayTestCase {
        section: Section,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct IonDisplayTestCase {
        ion: Ion,
        expected: &'static str,
    }

    fn string(value: &str) -> Value {
        Value::String(value.to_owned())
    }

    fn section(entries: Vec<(&str, Value)>, rows: Vec<Vec<Value>>) -> Section {
        let mut section = Section::new();
        for (key, value) in entries {
            section.dictionary.insert(key.to_owned(), value);
        }
        section.rows = rows;
        section
    }

    fn dictionary(entries: Vec<(&str, Value)>) -> Value {
        let mut dictionary = crate::Dictionary::new();
        for (key, value) in entries {
            dictionary.insert(key.to_owned(), value);
        }
        Value::Dictionary(dictionary)
    }

    static VALUE_DISPLAY_PLAIN_STRING: LazyLock<ValueDisplayTestCase> =
        LazyLock::new(|| ValueDisplayTestCase {
            value: string("a\nb\t|c"),
            alternate: false,
            expected: "a\\nb\\t\\|c",
        });
    static VALUE_DISPLAY_ALTERNATE_STRING: LazyLock<ValueDisplayTestCase> =
        LazyLock::new(|| ValueDisplayTestCase {
            value: string("a\\\n\"b"),
            alternate: true,
            expected: "\"a\\\\\\n\\\"b\"",
        });
    static VALUE_DISPLAY_EMPTY_ARRAY: LazyLock<ValueDisplayTestCase> =
        LazyLock::new(|| ValueDisplayTestCase {
            value: Value::Array(vec![]),
            alternate: true,
            expected: "[  ]",
        });
    static VALUE_DISPLAY_DICTIONARY: LazyLock<ValueDisplayTestCase> =
        LazyLock::new(|| ValueDisplayTestCase {
            value: dictionary(vec![("name", string("foo")), ("count", Value::Integer(2))]),
            alternate: true,
            expected: if cfg!(feature = "dictionary-indexmap") {
                "{ name = \"foo\", count = 2 }"
            } else {
                "{ count = 2, name = \"foo\" }"
            },
        });

    #[test_case(&*VALUE_DISPLAY_PLAIN_STRING; "plain string")]
    #[test_case(&*VALUE_DISPLAY_ALTERNATE_STRING; "alternate string")]
    #[test_case(&*VALUE_DISPLAY_EMPTY_ARRAY; "empty array")]
    #[test_case(&*VALUE_DISPLAY_DICTIONARY; "dictionary")]
    fn value_display(case: &ValueDisplayTestCase) {
        let actual = if case.alternate {
            format!("{:#}", case.value)
        } else {
            format!("{}", case.value)
        };
        assert_eq!(case.expected, actual);
    }

    static SECTION_DISPLAY_CASE: LazyLock<SectionDisplayTestCase> =
        LazyLock::new(|| SectionDisplayTestCase {
            section: section(
                vec![("name", string("foo"))],
                vec![vec![string("one"), Value::Integer(2)]],
            ),
            expected: indoc! {r#"
                name = "foo"
                | one | 2 |
            "#},
        });

    #[test_case(&*SECTION_DISPLAY_CASE; "section")]
    fn section_display(case: &SectionDisplayTestCase) {
        assert_eq!(case.expected, format!("{}", case.section));
    }

    static ION_DISPLAY_CASE: LazyLock<IonDisplayTestCase> = LazyLock::new(|| {
        let sections = std::collections::BTreeMap::from([
            (
                "ALPHA".to_owned(),
                section(vec![("name", string("foo"))], vec![]),
            ),
            (
                "BETA".to_owned(),
                section(vec![], vec![vec![string("one"), string("two")]]),
            ),
        ]);
        IonDisplayTestCase {
            ion: Ion::new(sections),
            expected: indoc! {r#"
                [ALPHA]
                name = "foo"

                [BETA]
                | one | two |

            "#},
        }
    });

    #[test_case(&*ION_DISPLAY_CASE; "ion")]
    fn ion_display(case: &IonDisplayTestCase) {
        assert_eq!(case.expected, format!("{}", case.ion));
    }
}
