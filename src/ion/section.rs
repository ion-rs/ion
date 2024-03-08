use crate::{Dictionary, FromIon, IonError, Row, Value};
use std::vec;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Section {
    pub dictionary: Dictionary,
    pub rows: Vec<Row>,
}

impl Section {
    pub fn new() -> Section {
        Self::with_capacity(1)
    }

    pub fn with_capacity(n: usize) -> Section {
        Self {
            dictionary: Dictionary::new(),
            rows: Vec::with_capacity(n),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.dictionary.get(name)
    }

    /// Returns a mutable reference to the field associated with the given name in the dictionary.
    ///
    /// If a field exists for the provided name, a mutable reference to that field is returned.
    /// If no field is associated with the name, `None` is returned.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.dictionary.get_mut(name)
    }

    pub fn fetch(&self, key: &str) -> Result<&Value, IonError> {
        self.get(key)
            .ok_or_else(|| IonError::MissingValue(key.to_owned()))
    }

    pub fn rows_without_header(&self) -> &[Row] {
        if self.rows.len() > 1 {
            let row = &self.rows[1];

            if row.first().map_or(false, |v| match v {
                Value::String(s) => !s.is_empty() && s.chars().all(|c| c == '-'),
                _ => false,
            }) {
                return &self.rows[2..];
            }
        }

        &self.rows
    }

    pub fn parse<F: FromIon<Section>>(&self) -> Result<F, F::Err> {
        F::from_ion(self)
    }
}

pub struct IntoIter<T> {
    iter: vec::IntoIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> IntoIterator for &'a Section {
    type Item = &'a Row;
    type IntoIter = std::slice::Iter<'a, Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows_without_header().iter()
    }
}

impl IntoIterator for Section {
    type Item = Row;
    type IntoIter = IntoIter<Row>;

    fn into_iter(self) -> Self::IntoIter {
        let has_header = self
            .rows
            .iter()
            .skip(1)
            .take(1)
            .take_while(|v| {
                if let Some(Value::String(s)) = v.get(1) {
                    s.starts_with('-')
                } else {
                    false
                }
            })
            .next()
            .is_some();

        if has_header {
            IntoIter {
                iter: self
                    .rows
                    .into_iter()
                    .skip(2)
                    .collect::<Vec<_>>()
                    .into_iter(),
            }
        } else {
            IntoIter {
                iter: self.rows.into_iter(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ion, Ion, Section};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use regex::Regex;

    fn is_input_string_invalid(s: &str) -> bool {
        Regex::new("[\n \t\r|\\\\]|^-+$").unwrap().is_match(s)
    }

    mod into_iter {
        use super::*;

        mod without_headers {
            use super::*;

            #[test]
            fn it_works_on_ref_section() {
                let ion = ion!(
                    r#"
                    [FOO]
                    |1||2|
                    |1|   |2|
                    |1|2|3|
                    "#
                );

                let section: &Section = ion.get("FOO").unwrap();
                let rows: Vec<_> = section.into_iter().collect();
                assert_eq!(3, rows.len());
            }

            #[test]
            fn it_works_on_section_by_value() {
                let mut ion = ion!(
                    r#"
                    [FOO]
                    |1||2|
                    |1|   |2|
                    |1|2|3|
                    "#
                );

                let section: Section = ion.remove("FOO").unwrap();
                let rows: Vec<_> = section.into_iter().collect();
                assert_eq!(3, rows.len());
            }

            #[test]
            fn it_works_with_loop() {
                let mut ion = ion!(
                    r#"
                    [FOO]
                    |1||2|
                    |1|   |2|
                    |1|2|3|
                    "#
                );

                let section: Section = ion.remove("FOO").unwrap();
                let mut rows = Vec::new();
                for row in section {
                    rows.push(row);
                }
                assert_eq!(3, rows.len());
            }
        }

        mod with_headers {
            use super::*;

            #[test]
            fn it_works_with_section_by_value() {
                let mut ion = ion!(
                    r#"
                    [FOO]
                    | 1 | 2 | 3 |
                    |---|---|---|
                    |1||2|
                    |1|   |2|
                    |1|2|3|
                    "#
                );

                let section: Section = ion.remove("FOO").unwrap();
                let rows: Vec<_> = section.into_iter().collect();

                assert_eq!(3, rows.len());
            }
        }
    }

    mod with_headers {
        use super::*;
        use crate::Value;

        #[quickcheck]
        fn works_for_any_arbitrary_cell_contents(item: String) -> TestResult {
            if is_input_string_invalid(item.as_str()) {
                return TestResult::discard();
            }

            let ion_str = format!(
                r#"
                [FOO]
                |head1|head2|head3|
                |-----|-----|-----|
                |{item}|{item}|{item}|
                |{item}|{item}|{item}|
                |{item}|{item}|{item}|
                "#,
            );

            let ion = ion_str.parse::<Ion>().unwrap();
            let section = ion.get("FOO").unwrap();

            TestResult::from_bool(3 == section.rows_without_header().len())
        }

        #[test]
        fn cell_content_can_start_with_hyphen() {
            let ion = ion!(
                r#"
                [FOO]
                |head1|head2|head3|
                |-----|-----|-----|
                | -3  | emp | a   |
                | -3  | -b  | b   |
                | -3  | b   | -b  |
                "#
            );

            let section = ion.get("FOO").unwrap();

            assert_eq!(3, section.rows_without_header().len())
        }

        #[test]
        fn cell_content_can_be_empty() {
            let ion = ion!(
                r#"
                [FOO]
                |head1|head2|head3|
                |-----|-----|-----|
                |     | emp | a   |
                |     |     | b   |
                |     | b   |     |
                "#
            );

            let section = ion.get("FOO").unwrap();

            assert_eq!(3, section.rows_without_header().len())
        }

        #[test]
        fn cell_content_with_escaped_pipe() {
            let ion = ion!(
                r#"
                [FOO]
                |head1 |head2 |head3 |head4 | head5  |
                |------|------|------|------|--------|
                | a\|b | a\\b | a\nb | a\tb | a\\\nb |
                "#
            );

            let section = ion.get("FOO").unwrap();
            let first_row = section.rows_without_header().first().unwrap();
            assert_eq!(5, first_row.len());
            assert_eq!(Value::String("a|b".to_string()), first_row[0]);
            assert_eq!(Value::String("a\\b".to_string()), first_row[1]);
            assert_eq!(Value::String("a\nb".to_string()), first_row[2]);
            assert_eq!(Value::String("a\tb".to_string()), first_row[3]);
            assert_eq!(Value::String("a\\\nb".to_string()), first_row[4]);
            assert_eq!(1, section.rows_without_header().len())
        }

        #[test]
        fn section_can_have_no_content_rows() {
            let ion = ion!(
                r#"
                [FOO]
                |head1|head2|head3|
                |-----|-----|-----|
                "#
            );

            let section = ion.get("FOO").unwrap();

            assert_eq!(0, section.rows_without_header().len())
        }
    }

    mod without_headers {
        use super::*;

        #[quickcheck]
        fn works_for_any_arbitrary_cell_contents(item: String) -> TestResult {
            if is_input_string_invalid(item.as_str()) {
                return TestResult::discard();
            }

            let ion_str = format!(
                r#"
                [FOO]
                |{item}|{item}|{item}|
                |{item}|{item}|{item}|
                |{item}|{item}|{item}|
                "#,
            );

            let ion = ion_str.parse::<Ion>().unwrap();
            let section = ion.get("FOO").unwrap();

            TestResult::from_bool(3 == section.rows_without_header().len())
        }

        #[test]
        fn cell_content_can_start_with_hyphen() {
            let ion = ion!(
                r#"
                [FOO]
                | -3  | emp | a   |
                | -3  | -b  | b   |
                | -3  | b   | -b  |
                "#
            );

            let section = ion.get("FOO").unwrap();

            assert_eq!(3, section.rows_without_header().len())
        }

        #[test]
        fn cell_content_can_be_empty() {
            let ion = ion!(
                r#"
                [FOO]
                |     | emp | a   |
                |     |     | b   |
                |     | b   |     |
                "#
            );

            let section = ion.get("FOO").unwrap();

            assert_eq!(3, section.rows_without_header().len())
        }

        #[test]
        fn cell_content_with_escaped_pipe() {
            let ion = ion!(
                r#"
                [FOO]
                |     | a\|b  | a   |
                |     |       | b   |
                |     | b     |     |
                "#
            );

            let section = ion.get("FOO").unwrap();
            let first_row = section.rows.first().unwrap();
            assert_eq!(3, first_row.len());
            assert_eq!("", first_row[0].to_string());
            assert_eq!("a|b", first_row[1].to_string());
            assert_eq!("a", first_row[2].to_string());
            assert_eq!(3, section.rows_without_header().len())
        }

        #[test]
        fn section_can_have_no_content_rows() {
            let ion = ion!(
                r#"
                [FOO]
                "#
            );

            let section = ion.get("FOO").unwrap();

            assert_eq!(0, section.rows_without_header().len())
        }
    }
}
