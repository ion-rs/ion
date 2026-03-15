use crate::{Dictionary, FromIon, IonError, Row, Value};
use std::vec;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Section {
    pub dictionary: Dictionary,
    pub rows: Vec<Row>,
}

impl Section {
    #[must_use]
    pub fn new() -> Section {
        Self::with_capacity(1)
    }

    #[must_use]
    pub fn with_capacity(n: usize) -> Section {
        Self {
            dictionary: Dictionary::new(),
            rows: Vec::with_capacity(n),
        }
    }

    #[must_use]
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

    /// # Errors
    ///
    /// Returns [`IonError::MissingValue`] when the key does not exist.
    pub fn fetch(&self, key: &str) -> Result<&Value, IonError> {
        self.get(key)
            .ok_or_else(|| IonError::MissingValue(key.to_owned()))
    }

    #[must_use]
    pub fn rows_without_header(&self) -> &[Row] {
        if self.rows.len() > 1 {
            let row = &self.rows[1];

            if row.first().is_some_and(|v| match v {
                Value::String(s) => !s.is_empty() && s.chars().all(|c| c == '-'),
                _ => false,
            }) {
                return &self.rows[2..];
            }
        }

        &self.rows
    }

    /// # Errors
    ///
    /// Returns any error produced by `F::from_ion`.
    pub fn parse<F: FromIon<Section>>(&self) -> Result<F, F::Err> {
        F::from_ion(self)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Row> {
        self.rows_without_header().iter()
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
        self.iter()
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
    use crate::{Dictionary, FromIon, Ion, IonError, Section, Value, ion};
    use pretty_assertions::assert_eq;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use regex::Regex;
    use std::sync::LazyLock;
    use test_case::test_case;

    fn is_input_string_invalid(s: &str) -> bool {
        Regex::new("[\n \t\r|\\\\]|^-+$").unwrap().is_match(s)
    }

    #[derive(Debug)]
    struct IntoIterTestCase {
        raw: &'static str,
        expected_rows: usize,
    }

    #[derive(Debug)]
    struct RowCountTestCase {
        raw: &'static str,
        expected_rows: usize,
    }

    #[derive(Debug)]
    struct EscapedCellTestCase {
        raw: &'static str,
        expected_first_row: Vec<Value>,
        expected_rows: usize,
        use_rows_without_header: bool,
    }

    #[derive(Debug)]
    struct SectionApiTestCase {
        section: Section,
        key: &'static str,
        expected_value: Option<Value>,
        expected_fetch_error: Option<&'static str>,
        expected_iter_len: usize,
    }

    struct ParsedSectionName(String);

    impl FromIon<Section> for ParsedSectionName {
        type Err = ();

        fn from_ion(section: &Section) -> Result<Self, Self::Err> {
            Ok(Self(
                section
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            ))
        }
    }

    static ROWS_NO_HEADER: &str = r"
        [FOO]
        |1||2|
        |1|   |2|
        |1|2|3|
    ";
    const INTO_ITER_REF_CASE: IntoIterTestCase = IntoIterTestCase {
        raw: ROWS_NO_HEADER,
        expected_rows: 3,
    };
    const INTO_ITER_VALUE_CASE: IntoIterTestCase = IntoIterTestCase {
        raw: ROWS_NO_HEADER,
        expected_rows: 3,
    };
    const INTO_ITER_LOOP_CASE: IntoIterTestCase = IntoIterTestCase {
        raw: ROWS_NO_HEADER,
        expected_rows: 3,
    };
    const INTO_ITER_WITH_HEADER_CASE: IntoIterTestCase = IntoIterTestCase {
        raw: r"
                [FOO]
                | 1 | 2 | 3 |
                |---|---|---|
                |1||2|
                |1|   |2|
                |1|2|3|
            ",
        expected_rows: 3,
    };

    #[test_case(&INTO_ITER_REF_CASE; "ref section without headers")]
    fn into_iter_ref(case: &IntoIterTestCase) {
        let ion = ion!(case.raw);
        let section: &Section = ion.get("FOO").unwrap();
        let rows: Vec<_> = section.into_iter().collect();
        assert_eq!(case.expected_rows, rows.len());
    }

    #[test_case(&INTO_ITER_VALUE_CASE; "owned section without headers")]
    fn into_iter_value(case: &IntoIterTestCase) {
        let mut ion = ion!(case.raw);
        let section: Section = ion.remove("FOO").unwrap();
        let rows: Vec<_> = section.into_iter().collect();
        assert_eq!(case.expected_rows, rows.len());
    }

    #[test_case(&INTO_ITER_LOOP_CASE; "loop without headers")]
    fn into_iter_loop(case: &IntoIterTestCase) {
        let mut ion = ion!(case.raw);
        let section: Section = ion.remove("FOO").unwrap();
        let mut rows = Vec::new();
        for row in section {
            rows.push(row);
        }
        assert_eq!(case.expected_rows, rows.len());
    }

    #[test_case(&INTO_ITER_WITH_HEADER_CASE; "owned section with header")]
    fn into_iter_with_headers(case: &IntoIterTestCase) {
        let mut ion = ion!(case.raw);
        let section: Section = ion.remove("FOO").unwrap();
        let rows: Vec<_> = section.into_iter().collect();
        assert_eq!(case.expected_rows, rows.len());
    }

    #[quickcheck]
    fn with_headers_works_for_any_arbitrary_cell_contents(item: String) -> TestResult {
        if is_input_string_invalid(&item) {
            return TestResult::discard();
        }
        let item = item.into_boxed_str();

        let ion_str = format!(
            r"
            [FOO]
            |head1|head2|head3|
            |-----|-----|-----|
            |{item}|{item}|{item}|
            |{item}|{item}|{item}|
            |{item}|{item}|{item}|
            ",
        );

        let ion = ion_str.parse::<Ion>().unwrap();
        let section = ion.get("FOO").unwrap();

        TestResult::from_bool(section.rows_without_header().len() == 3)
    }

    const WITH_HEADERS_HYPHEN_CASE: RowCountTestCase = RowCountTestCase {
        raw: r"
                [FOO]
                |head1|head2|head3|
                |-----|-----|-----|
                | -3  | emp | a   |
                | -3  | -b  | b   |
                | -3  | b   | -b  |
            ",
        expected_rows: 3,
    };
    const WITH_HEADERS_EMPTY_CASE: RowCountTestCase = RowCountTestCase {
        raw: r"
                [FOO]
                |head1|head2|head3|
                |-----|-----|-----|
                |     | emp | a   |
                |     |     | b   |
                |     | b   |     |
            ",
        expected_rows: 3,
    };
    const WITH_HEADERS_NO_ROWS_CASE: RowCountTestCase = RowCountTestCase {
        raw: r"
                [FOO]
                |head1|head2|head3|
                |-----|-----|-----|
            ",
        expected_rows: 0,
    };
    const WITHOUT_HEADERS_HYPHEN_CASE: RowCountTestCase = RowCountTestCase {
        raw: r"
                [FOO]
                | -3  | emp | a   |
                | -3  | -b  | b   |
                | -3  | b   | -b  |
            ",
        expected_rows: 3,
    };
    const WITHOUT_HEADERS_EMPTY_CASE: RowCountTestCase = RowCountTestCase {
        raw: r"
                [FOO]
                |     | emp | a   |
                |     |     | b   |
                |     | b   |     |
            ",
        expected_rows: 3,
    };
    const WITHOUT_HEADERS_NO_ROWS_CASE: RowCountTestCase = RowCountTestCase {
        raw: r"
                [FOO]
            ",
        expected_rows: 0,
    };

    #[test_case(&WITH_HEADERS_HYPHEN_CASE; "with headers hyphen")]
    #[test_case(&WITH_HEADERS_EMPTY_CASE; "with headers empty")]
    #[test_case(&WITH_HEADERS_NO_ROWS_CASE; "with headers no rows")]
    #[test_case(&WITHOUT_HEADERS_HYPHEN_CASE; "without headers hyphen")]
    #[test_case(&WITHOUT_HEADERS_EMPTY_CASE; "without headers empty")]
    #[test_case(&WITHOUT_HEADERS_NO_ROWS_CASE; "without headers no rows")]
    fn rows_without_header_counts(case: &RowCountTestCase) {
        let ion = ion!(case.raw);
        let section = ion.get("FOO").unwrap();
        assert_eq!(case.expected_rows, section.rows_without_header().len());
    }

    static WITH_HEADERS_ESCAPED_CASE: LazyLock<EscapedCellTestCase> =
        LazyLock::new(|| EscapedCellTestCase {
            raw: r"
                [FOO]
                |head1 |head2 |head3 |head4 | head5  |
                |------|------|------|------|--------|
                | a\|b | a\\b | a\nb | a\tb | a\\\nb |
            ",
            expected_first_row: vec![
                Value::String("a|b".to_owned()),
                Value::String("a\\b".to_owned()),
                Value::String("a\nb".to_owned()),
                Value::String("a\tb".to_owned()),
                Value::String("a\\\nb".to_owned()),
            ],
            expected_rows: 1,
            use_rows_without_header: true,
        });
    static WITHOUT_HEADERS_ESCAPED_CASE: LazyLock<EscapedCellTestCase> =
        LazyLock::new(|| EscapedCellTestCase {
            raw: r"
                [FOO]
                |     | a\|b  | a   |
                |     |       | b   |
                |     | b     |     |
            ",
            expected_first_row: vec![
                Value::String(String::new()),
                Value::String("a|b".to_owned()),
                Value::String("a".to_owned()),
            ],
            expected_rows: 3,
            use_rows_without_header: false,
        });

    #[test_case(&*WITH_HEADERS_ESCAPED_CASE; "with headers escaped pipe")]
    #[test_case(&*WITHOUT_HEADERS_ESCAPED_CASE; "without headers escaped pipe")]
    fn escaped_cells(case: &EscapedCellTestCase) {
        let ion = ion!(case.raw);
        let section = ion.get("FOO").unwrap();
        let first_row = if case.use_rows_without_header {
            section.rows_without_header().first().unwrap()
        } else {
            section.rows.first().unwrap()
        };

        assert_eq!(case.expected_first_row.len(), first_row.len());
        assert_eq!(&case.expected_first_row, first_row);
        assert_eq!(case.expected_rows, section.rows_without_header().len());
    }

    static SECTION_API_PRESENT_CASE: LazyLock<SectionApiTestCase> = LazyLock::new(|| {
        let mut section = Section::with_capacity(2);
        section
            .dictionary
            .insert("name".to_owned(), Value::new_string("foo"));
        section.rows = vec![
            vec![Value::new_string("h1")],
            vec![Value::new_string("---")],
            vec![Value::new_string("row")],
        ];
        SectionApiTestCase {
            section,
            key: "name",
            expected_value: Some(Value::new_string("foo")),
            expected_fetch_error: None,
            expected_iter_len: 1,
        }
    });
    static SECTION_API_MISSING_CASE: LazyLock<SectionApiTestCase> = LazyLock::new(|| {
        let mut section = Section::new();
        section.rows = vec![vec![Value::new_string("row")]];
        SectionApiTestCase {
            section,
            key: "missing",
            expected_value: None,
            expected_fetch_error: Some("missing"),
            expected_iter_len: 1,
        }
    });

    #[test_case(&*SECTION_API_PRESENT_CASE; "section api present")]
    #[test_case(&*SECTION_API_MISSING_CASE; "section api missing")]
    fn section_api(case: &SectionApiTestCase) {
        assert_eq!(case.expected_value.as_ref(), case.section.get(case.key));

        let fetch = case.section.fetch(case.key);
        match case.expected_fetch_error {
            Some(expected) => match fetch {
                Err(IonError::MissingValue(actual)) => assert_eq!(expected, actual),
                other => panic!("unexpected fetch result: {other:?}"),
            },
            None => assert_eq!(case.expected_value.as_ref(), fetch.ok()),
        }

        let iter_rows: Vec<_> = case.section.iter().collect();
        assert_eq!(case.expected_iter_len, iter_rows.len());
    }

    #[test]
    fn section_get_mut_and_parse() {
        let mut section = Section::new();
        section
            .dictionary
            .insert("name".to_owned(), Value::new_string("foo"));

        match section.get_mut("name") {
            Some(Value::String(value)) => *value = "bar".to_owned(),
            other => panic!("unexpected mutable value: {other:?}"),
        }

        assert_eq!(Some("bar"), section.get("name").and_then(Value::as_str));
        let parsed: ParsedSectionName = section.parse().unwrap();
        assert_eq!("bar", parsed.0);
    }

    #[test]
    fn rows_without_header_requires_a_hyphen_only_row() {
        let section = Section {
            dictionary: Dictionary::default(),
            rows: vec![
                vec![Value::new_string("h1"), Value::new_string("h2")],
                vec![Value::Integer(1), Value::new_string("---")],
                vec![Value::new_string("row"), Value::new_string("value")],
            ],
        };

        assert_eq!(3, section.rows_without_header().len());
    }

    #[quickcheck]
    fn without_headers_works_for_any_arbitrary_cell_contents(item: String) -> TestResult {
        if is_input_string_invalid(&item) {
            return TestResult::discard();
        }
        let item = item.into_boxed_str();

        let ion_str = format!(
            r"
            [FOO]
            |{item}|{item}|{item}|
            |{item}|{item}|{item}|
            |{item}|{item}|{item}|
            ",
        );

        let ion = ion_str.parse::<Ion>().unwrap();
        let section = ion.get("FOO").unwrap();

        TestResult::from_bool(section.rows_without_header().len() == 3)
    }
}
