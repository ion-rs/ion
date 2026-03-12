use crate::{Section, Value};
use std::collections::BTreeMap;
use std::iter::Peekable;
use std::{error, fmt, str};

#[derive(Debug, PartialEq)]
pub enum Element {
    Section(String),
    Row(Vec<Value>),
    Entry(String, Value),
    Comment(String),
}

pub struct Parser<'a> {
    input: &'a str,
    cur: Peekable<str::CharIndices<'a>>,
    pub(crate) errors: Vec<ParserError>,
    accepted_sections: Option<Vec<&'a str>>,
    section_capacity: usize,
    row_capacity: usize,
    array_capacity: usize,
}

impl Iterator for Parser<'_> {
    type Item = Element;

    fn next(&mut self) -> Option<Element> {
        let mut is_section_accepted = true;

        loop {
            self.whitespace();

            if self.newline() {
                continue;
            }

            let c = match self.cur.peek() {
                Some((_, c)) => *c,
                None => return None,
            };

            if c == '[' {
                let name = self.section_name();

                match self.is_section_accepted(&name) {
                    Some(true) => return Some(Element::Section(name)),
                    Some(false) => is_section_accepted = false,
                    None => return None,
                }
            }

            if !is_section_accepted {
                self.skip_line();
                continue;
            }

            return match c {
                '|' => Some(self.row()),
                '#' => self.comment(),
                _ => self.entry(),
            };
        }
    }
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(s: &'a str) -> Self {
        Self::new_filtered_opt(s, None)
    }

    #[must_use]
    pub fn new_filtered(s: &'a str, accepted_sections: Vec<&'a str>) -> Self {
        Self::new_filtered_opt(s, Some(accepted_sections))
    }

    #[must_use]
    pub fn with_section_capacity(mut self, section_capacity: usize) -> Self {
        self.section_capacity = section_capacity;
        self
    }

    #[must_use]
    pub fn with_row_capacity(mut self, row_capacity: usize) -> Self {
        self.row_capacity = row_capacity;
        self
    }

    #[must_use]
    pub fn with_array_capacity(mut self, array_capacity: usize) -> Self {
        self.array_capacity = array_capacity;
        self
    }

    fn new_filtered_opt(s: &'a str, accepted_sections: Option<Vec<&'a str>>) -> Self {
        Self {
            input: s,
            cur: s.char_indices().peekable(),
            errors: Vec::new(),
            accepted_sections,
            section_capacity: 16,
            row_capacity: 8,
            array_capacity: 2,
        }
    }

    fn whitespace(&mut self) {
        while let Some((_, '\t' | ' ')) = self.cur.peek() {
            self.cur.next();
        }
    }

    fn newline(&mut self) -> bool {
        match self.cur.peek() {
            Some((_, '\n')) => {
                self.cur.next();
                true
            }

            Some((_, '\r')) => {
                self.cur.next();
                if let Some((_, '\n')) = self.cur.peek() {
                    self.cur.next();
                }
                true
            }

            _ => false,
        }
    }

    fn skip_line(&mut self) {
        self.cur.by_ref().find(|&(_, c)| c != '\n');
    }

    fn comment(&mut self) -> Option<Element> {
        if !self.eat('#') {
            return None;
        }

        Some(Element::Comment(
            self.slice_to_including('\n').unwrap_or("").to_string(),
        ))
    }

    fn eat(&mut self, ch: char) -> bool {
        match self.cur.peek() {
            Some((_, c)) if *c == ch => {
                self.cur.next();
                true
            }
            _ => false,
        }
    }

    fn section_name(&mut self) -> String {
        self.eat('[');
        self.whitespace();

        self.cur
            .by_ref()
            .map(|(_, c)| c)
            .take_while(|c| *c != ']')
            .collect()
    }

    fn entry(&mut self) -> Option<Element> {
        if let Some(key) = self.key_name() {
            if !self.keyval_sep() {
                return None;
            }

            if let Some(val) = self.value() {
                return Some(Element::Entry(key, val));
            }
        }

        None
    }

    fn key_name(&mut self) -> Option<String> {
        self.slice_while(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-'))
            .map(str::to_owned)
    }

    fn value(&mut self) -> Option<Value> {
        self.whitespace();
        self.newline();
        self.whitespace();

        match self.cur.peek() {
            Some((_, '"')) => self.finish_string(),
            Some((_, '[')) => self.finish_array(),
            Some((_, '{')) => self.finish_dictionary(),
            Some((_, '-')) => self.number(),
            Some((_, ch)) if ch.is_ascii_digit() => self.number(),
            Some((pos, 't' | 'f')) => {
                let pos = *pos;
                self.boolean(pos)
            }
            _ => {
                self.add_error("Cannot read a value");
                None
            }
        }
    }

    fn finish_array(&mut self) -> Option<Value> {
        self.cur.next();

        let mut row = Vec::with_capacity(self.array_capacity);

        loop {
            self.whitespace();

            if let Some((_, ch)) = self.cur.peek() {
                match ch {
                    ']' => {
                        self.cur.next();
                        return Some(Value::Array(row));
                    }
                    ',' => {
                        self.cur.next();
                    }
                    _ => match self.value() {
                        Some(v) => row.push(v),
                        None => break,
                    },
                }
            } else {
                self.add_error("Cannot finish an array");
                break;
            }
        }

        None
    }

    fn finish_dictionary(&mut self) -> Option<Value> {
        self.cur.next();
        let mut map = BTreeMap::new();

        loop {
            self.whitespace();

            if let Some((_, ch)) = self.cur.peek() {
                match ch {
                    '}' => {
                        self.cur.next();
                        return Some(Value::Dictionary(map));
                    }
                    ',' | '\n' => {
                        self.cur.next();
                    }
                    _ => {
                        match self.entry() {
                            Some(Element::Entry(k, v)) => map.insert(k, v),
                            None => break,
                            _ => panic!("Element::Entry expected"),
                        };
                    }
                }
            } else {
                self.add_error("Cannot finish a dictionary");
                break;
            }
        }

        None
    }

    fn number(&mut self) -> Option<Value> {
        let mut is_float = false;
        let prefix = self.integer()?;

        let decimal = if self.eat('.') {
            is_float = true;
            Some(self.integer())?
        } else {
            None
        };

        let input = match &decimal {
            Some(decimal) => prefix + "." + decimal,
            None => prefix,
        };

        if is_float {
            input.parse().ok().map(Value::Float)
        } else {
            input.parse().ok().map(Value::Integer)
        }
    }

    fn integer(&mut self) -> Option<String> {
        // read optional leading '-' and digits until non-digit is encountered
        self.slice_while(|ch| ch == '-' || ch.is_ascii_digit())
            .map(str::to_owned)
    }

    fn boolean(&mut self, start: usize) -> Option<Value> {
        let rest = &self.input[start..];

        if rest.starts_with("true") {
            for _ in 0..4 {
                self.cur.next();
            }

            Some(Value::Boolean(true))
        } else if rest.starts_with("false") {
            for _ in 0..5 {
                self.cur.next();
            }

            Some(Value::Boolean(false))
        } else {
            None
        }
    }

    fn finish_string(&mut self) -> Option<Value> {
        self.cur.next();

        self.slice_to_excluding('"')
            .map(|s| Value::String(replace_escapes(s, true)))
    }

    fn keyval_sep(&mut self) -> bool {
        self.whitespace();

        if !self.expect('=') {
            return false;
        }

        self.whitespace();
        true
    }

    fn expect(&mut self, ch: char) -> bool {
        self.eat(ch)
    }

    fn row(&mut self) -> Element {
        let mut row = Vec::with_capacity(self.row_capacity);

        self.eat('|');

        loop {
            self.whitespace();

            if self.comment().is_some() {
                break;
            }

            if self.newline() {
                break;
            }

            if self.cur.peek().is_none() {
                break;
            }

            row.push(Value::String(self.cell()));
        }

        Element::Row(row)
    }

    fn cell(&mut self) -> String {
        self.whitespace();

        replace_escapes(
            self.slice_to_excluding('|')
                .map(str::trim_end)
                .unwrap_or_default(),
            false,
        )
    }

    pub fn read(&mut self) -> Option<BTreeMap<String, Section>> {
        let mut map = BTreeMap::new();
        let mut section = Section::with_capacity(self.section_capacity);
        let mut name = None;

        while let Some(el) = self.next() {
            match el {
                Element::Section(n) => {
                    if let Some(name) = name {
                        map.insert(name, section);
                    }
                    name = Some(n);
                    section = Section::with_capacity(self.section_capacity);
                }
                Element::Row(row) => section.rows.push(row),
                Element::Entry(key, value) => {
                    section.dictionary.insert(key, value);
                }
                Element::Comment(_) => {}
            }
        }

        match name {
            Some(name) => {
                map.insert(name, section);
            }
            None if self.accepted_sections.is_none() => {
                map.insert("root".to_string(), section);
            }
            _ => (),
        }

        if self.errors.is_empty() {
            Some(map)
        } else {
            None
        }
    }

    fn is_section_accepted(&mut self, name: &str) -> Option<bool> {
        let Some(sections) = &mut self.accepted_sections else {
            return Some(true);
        };

        if sections.is_empty() {
            return None;
        }

        match sections.iter().position(|s| *s == name) {
            Some(idx) => {
                sections.swap_remove(idx);
                Some(true)
            }
            None => Some(false),
        }
    }

    fn slice_to_including(&mut self, ch: char) -> Option<&str> {
        self.cur.next().map(|(start, c)| {
            if c == ch {
                &self.input[start..=start]
            } else {
                self.cur
                    .find(|(_, c)| *c == ch)
                    .map_or(&self.input[start..], |(end, _)| &self.input[start..=end])
            }
        })
    }

    fn slice_to_excluding(&mut self, ch: char) -> Option<&str> {
        self.cur.next().map(|(start, c)| {
            if c == ch {
                ""
            } else {
                let mut prev_element = c;

                for (i, cur_ch) in self.cur.by_ref() {
                    if cur_ch == ch && prev_element != '\\' {
                        return &self.input[start..i];
                    }

                    prev_element = cur_ch;
                }

                &self.input[start..]
            }
        })
    }

    fn slice_while(&mut self, predicate: impl Fn(char) -> bool) -> Option<&str> {
        self.cur.peek().copied().and_then(|(start, c)| {
            if predicate(c) {
                self.cur.next();

                while let Some(&(end, c)) = self.cur.peek() {
                    if !predicate(c) {
                        return Some(&self.input[start..end]);
                    }

                    self.cur.next();
                }

                Some(&self.input[start..])
            } else {
                None
            }
        })
    }

    fn add_error(&mut self, message: &str) {
        let mut it = self.cur.clone();
        let lo = it.next().map_or(self.input.len(), |p| p.0);
        let hi = it.next().map_or(self.input.len(), |p| p.0);

        self.errors.push(ParserError {
            lo,
            hi,
            desc: message.to_owned(),
        });
    }
}

#[derive(Clone, Debug)]
pub struct ParserError {
    pub lo: usize,
    pub hi: usize,
    pub desc: String,
}

impl error::Error for ParserError {
    fn description(&self) -> &'static str {
        "error parsing Ion"
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

fn replace_escapes(s: &str, escape_quote: bool) -> String {
    let mut result = String::new();
    let mut escaping = false;
    for c in s.chars() {
        match (escaping, c) {
            (false, '\\') => {
                escaping = true;
                continue;
            }
            (false, c) => result.push(c),

            (true, 'n') => result.push('\n'),
            (true, 't') => result.push('\t'),
            (true, '\\' | '|') => result.push(c),
            (true, '"') if escape_quote => result.push(c),
            (true, c) => {
                // When an unknown escape is encountered, print it as is e.g. \a -> \a
                result.push('\\');
                result.push(c);
            }
        }
        escaping = false;
    }

    // handle '\\' as last char in sequence
    if escaping {
        result.push('\\');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::Element::{self, Comment, Entry, Row};
    use crate::{Dictionary, Parser, Section, Value};
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct FinishStringTestCase {
        raw: &'static str,
        expected: Option<&'static str>,
    }

    #[derive(Debug)]
    struct FinishValueTestCase {
        raw: &'static str,
        expected: Option<Value>,
    }

    #[derive(Debug)]
    struct SliceTargetTestCase {
        raw: &'static str,
        target: char,
        expected: Option<&'static str>,
        next: Option<(usize, char)>,
    }

    #[derive(Debug)]
    struct SliceWhileTestCase {
        raw: &'static str,
        stop_at: char,
        expected: Option<&'static str>,
        next: Option<(usize, char)>,
    }

    #[derive(Debug)]
    struct ParseIteratorTestCase {
        raw: &'static str,
        expected: Vec<Element>,
    }

    #[derive(Debug)]
    struct DisplayTestCase {
        value: Value,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct ReplaceEscapesTestCase {
        raw: &'static str,
        escape_quote: bool,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct ReadTestCase {
        raw: &'static str,
        accepted_sections: &'static [&'static str],
        expected: Option<BTreeMap<String, Section>>,
    }

    fn string(value: &str) -> Value {
        Value::String(value.to_owned())
    }

    fn array(values: Vec<Value>) -> Value {
        Value::Array(values)
    }

    fn dictionary(entries: Vec<(&str, Value)>) -> Value {
        let mut dictionary = Dictionary::new();
        for (key, value) in entries {
            dictionary.insert(key.to_owned(), value);
        }
        Value::Dictionary(dictionary)
    }

    fn row(values: &[&str]) -> Vec<Value> {
        values.iter().map(|value| string(value)).collect()
    }

    fn section(entries: Vec<(&str, Value)>, rows: Vec<Vec<Value>>) -> Section {
        let mut section = Section::new();
        for (key, value) in entries {
            section.dictionary.insert(key.to_owned(), value);
        }
        section.rows = rows;
        section
    }

    fn sections(entries: Vec<(&str, Section)>) -> BTreeMap<String, Section> {
        let mut sections = BTreeMap::new();
        for (name, section) in entries {
            sections.insert(name.to_owned(), section);
        }
        sections
    }

    static FINISH_STRING_COMPLETE: LazyLock<FinishStringTestCase> =
        LazyLock::new(|| FinishStringTestCase {
            raw: "\"foObar\"",
            expected: Some("foObar"),
        });
    static FINISH_STRING_UNTERMINATED: LazyLock<FinishStringTestCase> =
        LazyLock::new(|| FinishStringTestCase {
            raw: "\"foObar",
            expected: Some("foObar"),
        });
    static FINISH_STRING_EMPTY: LazyLock<FinishStringTestCase> =
        LazyLock::new(|| FinishStringTestCase {
            raw: "\"\"",
            expected: Some(""),
        });
    static FINISH_STRING_MISSING: LazyLock<FinishStringTestCase> =
        LazyLock::new(|| FinishStringTestCase {
            raw: "",
            expected: None,
        });

    static FINISH_ARRAY_UNTERMINATED_VALUE: LazyLock<FinishValueTestCase> =
        LazyLock::new(|| FinishValueTestCase {
            raw: "[\"a\"",
            expected: None,
        });
    static FINISH_ARRAY_MISSING_CLOSE: LazyLock<FinishValueTestCase> =
        LazyLock::new(|| FinishValueTestCase {
            raw: "[",
            expected: None,
        });
    static FINISH_ARRAY_EMPTY: LazyLock<FinishValueTestCase> =
        LazyLock::new(|| FinishValueTestCase {
            raw: "[]",
            expected: Some(array(vec![])),
        });
    static FINISH_ARRAY_SINGLE_VALUE: LazyLock<FinishValueTestCase> =
        LazyLock::new(|| FinishValueTestCase {
            raw: "[\"a\"]",
            expected: Some(Value::new_string_array("a")),
        });

    static FINISH_DICTIONARY_MISSING_CLOSE: LazyLock<FinishValueTestCase> =
        LazyLock::new(|| FinishValueTestCase {
            raw: "{",
            expected: None,
        });
    static FINISH_DICTIONARY_MISSING_ASSIGNMENT_VALUE: LazyLock<FinishValueTestCase> =
        LazyLock::new(|| FinishValueTestCase {
            raw: "{ foo = ",
            expected: None,
        });
    static FINISH_DICTIONARY_UNTERMINATED_ARRAY: LazyLock<FinishValueTestCase> =
        LazyLock::new(|| FinishValueTestCase {
            raw: "{ foo = [\"bar\"]",
            expected: None,
        });
    static FINISH_DICTIONARY_EMPTY: LazyLock<FinishValueTestCase> =
        LazyLock::new(|| FinishValueTestCase {
            raw: "{}",
            expected: Some(dictionary(vec![])),
        });
    static FINISH_DICTIONARY_WITH_ARRAY: LazyLock<FinishValueTestCase> =
        LazyLock::new(|| FinishValueTestCase {
            raw: "{ foo = [\"bar\"] }",
            expected: Some(dictionary(vec![("foo", array(vec![string("bar")]))])),
        });

    static SLICE_TO_INCLUDING_END: LazyLock<SliceTargetTestCase> =
        LazyLock::new(|| SliceTargetTestCase {
            raw: "foObar",
            target: 'b',
            expected: Some("foOb"),
            next: Some((4, 'a')),
        });
    static SLICE_TO_INCLUDING_START: LazyLock<SliceTargetTestCase> =
        LazyLock::new(|| SliceTargetTestCase {
            raw: "foObar",
            target: 'f',
            expected: Some("f"),
            next: Some((1, 'o')),
        });

    static SLICE_TO_EXCLUDING_END: LazyLock<SliceTargetTestCase> =
        LazyLock::new(|| SliceTargetTestCase {
            raw: "foObar",
            target: 'b',
            expected: Some("foO"),
            next: Some((4, 'a')),
        });
    static SLICE_TO_EXCLUDING_START: LazyLock<SliceTargetTestCase> =
        LazyLock::new(|| SliceTargetTestCase {
            raw: "foObar",
            target: 'f',
            expected: Some(""),
            next: Some((1, 'o')),
        });
    static SLICE_TO_EXCLUDING_ESCAPED: LazyLock<SliceTargetTestCase> =
        LazyLock::new(|| SliceTargetTestCase {
            raw: "f\\oobar",
            target: 'o',
            expected: Some("f\\o"),
            next: Some((4, 'b')),
        });

    static SLICE_WHILE_UNTIL_MATCH: LazyLock<SliceWhileTestCase> =
        LazyLock::new(|| SliceWhileTestCase {
            raw: "foObar",
            stop_at: 'b',
            expected: Some("foO"),
            next: Some((3, 'b')),
        });
    static SLICE_WHILE_STOPS_IMMEDIATELY: LazyLock<SliceWhileTestCase> =
        LazyLock::new(|| SliceWhileTestCase {
            raw: "foObar",
            stop_at: 'f',
            expected: None,
            next: Some((0, 'f')),
        });

    static PARSE_MAIN_CASE: LazyLock<ParseIteratorTestCase> =
        LazyLock::new(|| ParseIteratorTestCase {
            raw: r#"
                [dict]
                first = "first"
                # comment
                second ="another"
                whitespace = "  "
                empty = ""
                some_bool = true

                ary = [ "col1", 2,"col3", false]

                [table]

                |abc|def|
                |---|---|
                |one|two|
                # comment
                |  1| 2 |
                |  2| 3 |

                [three]
                a=1
                B=2
                | this |
            "#,
            expected: vec![
                Element::Section("dict".to_owned()),
                Entry("first".to_owned(), string("first")),
                Comment(" comment\n".to_owned()),
                Entry("second".to_owned(), string("another")),
                Entry("whitespace".to_owned(), string("  ")),
                Entry("empty".to_owned(), string("")),
                Entry("some_bool".to_owned(), Value::Boolean(true)),
                Entry(
                    "ary".to_owned(),
                    array(vec![
                        string("col1"),
                        Value::Integer(2),
                        string("col3"),
                        Value::Boolean(false),
                    ]),
                ),
                Element::Section("table".to_owned()),
                Row(row(&["abc", "def"])),
                Row(row(&["---", "---"])),
                Row(row(&["one", "two"])),
                Comment(" comment\n".to_owned()),
                Row(row(&["1", "2"])),
                Row(row(&["2", "3"])),
                Element::Section("three".to_owned()),
                Entry("a".to_owned(), Value::Integer(1)),
                Entry("B".to_owned(), Value::Integer(2)),
                Row(row(&["this"])),
            ],
        });

    static DISPLAY_ARRAY: LazyLock<DisplayTestCase> = LazyLock::new(|| DisplayTestCase {
        value: array(vec![Value::Integer(1), string("foo")]),
        expected: "[ 1, \"foo\" ]",
    });

    static REPLACE_ESCAPES_PLAIN_TEXT: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: "a b",
            escape_quote: true,
            expected: "a b",
        });
    static REPLACE_ESCAPES_TRAILING_SLASH: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: r"a b\",
            escape_quote: true,
            expected: "a b\\",
        });
    static REPLACE_ESCAPES_NEWLINE: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: r"a\nb",
            escape_quote: true,
            expected: "a\nb",
        });
    static REPLACE_ESCAPES_TAB: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: r"a\tb",
            escape_quote: true,
            expected: "a\tb",
        });
    static REPLACE_ESCAPES_BACKSLASH: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: r"a\\b",
            escape_quote: true,
            expected: r"a\b",
        });
    static REPLACE_ESCAPES_LITERAL_SEQUENCE: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: r"a\\nb",
            escape_quote: true,
            expected: r"a\nb",
        });
    static REPLACE_ESCAPES_PIPE: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: r"a\|b",
            escape_quote: true,
            expected: "a|b",
        });
    static REPLACE_ESCAPES_QUOTE_ESCAPED: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: "a\\\"b",
            escape_quote: true,
            expected: "a\"b",
        });
    static REPLACE_ESCAPES_QUOTE_LITERAL: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: "a\\\"b",
            escape_quote: false,
            expected: "a\\\"b",
        });
    static REPLACE_ESCAPES_UNKNOWN_ESCAPES: LazyLock<ReplaceEscapesTestCase> =
        LazyLock::new(|| ReplaceEscapesTestCase {
            raw: r"a\\n\\t\\\b",
            escape_quote: true,
            expected: r"a\n\t\\b",
        });

    static READ_ROOT_STRING: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r#"
            foo = "bar"
        "#,
        accepted_sections: &[],
        expected: Some(sections(vec![(
            "root",
            section(vec![("foo", string("bar"))], vec![]),
        )])),
    });
    static READ_ROOT_ARRAY: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r#"
            arr = ["WAW", "WRO"]
        "#,
        accepted_sections: &[],
        expected: Some(sections(vec![(
            "root",
            section(
                vec![("arr", array(vec![string("WAW"), string("WRO")]))],
                vec![],
            ),
        )])),
    });
    static READ_ROOT_DICTIONARY: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r#"
            ndict = { foo = "bar" }
        "#,
        accepted_sections: &[],
        expected: Some(sections(vec![(
            "root",
            section(
                vec![("ndict", dictionary(vec![("foo", string("bar"))]))],
                vec![],
            ),
        )])),
    });
    static READ_ROOT_MULTILINE_DICTIONARY: LazyLock<ReadTestCase> =
        LazyLock::new(|| ReadTestCase {
            raw: r#"
                R75042 = {
                view = "SV"
                loc  = ["M", "B"]
                dist = { beach_km = 4.1 }
            }"#,
            accepted_sections: &[],
            expected: Some(sections(vec![(
                "root",
                section(
                    vec![(
                        "R75042",
                        dictionary(vec![
                            ("view", string("SV")),
                            ("loc", array(vec![string("M"), string("B")])),
                            ("dist", dictionary(vec![("beach_km", Value::Float(4.1))])),
                        ]),
                    )],
                    vec![],
                ),
            )])),
        });
    static READ_ROOT_MISSING_VALUE: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r"
            key =
        ",
        accepted_sections: &[],
        expected: None,
    });
    static READ_ROOT_ROWS: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r"
            |1|2|
            |3|
        ",
        accepted_sections: &[],
        expected: Some(sections(vec![(
            "root",
            section(vec![], vec![row(&["1", "2"]), row(&["3"])]),
        )])),
    });
    static READ_ROOT_ROWS_WITH_EMPTY_CELLS: LazyLock<ReadTestCase> =
        LazyLock::new(|| ReadTestCase {
            raw: r"
                |1||2|
                |3|   |
            ",
            accepted_sections: &[],
            expected: Some(sections(vec![(
                "root",
                section(
                    vec![],
                    vec![
                        vec![string("1"), string(""), string("2")],
                        vec![string("3"), string("")],
                    ],
                ),
            )])),
        });
    static READ_ROOT_NEGATIVE_NUMBERS: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r"
            fee_negated = -10.00
            discount = -5
        ",
        accepted_sections: &[],
        expected: Some(sections(vec![(
            "root",
            section(
                vec![
                    ("fee_negated", Value::Float(-10.0)),
                    ("discount", Value::Integer(-5)),
                ],
                vec![],
            ),
        )])),
    });
    static READ_SECTION_ONCE: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r#"
            [SECTION]

            key = "value"
            # now a table
            | col1 | col2|
            | col1 | col2| # comment
            | col1 | col2|
        "#,
        accepted_sections: &[],
        expected: Some(sections(vec![(
            "SECTION",
            section(
                vec![("key", string("value"))],
                vec![
                    row(&["col1", "col2"]),
                    row(&["col1", "col2"]),
                    row(&["col1", "col2"]),
                ],
            ),
        )])),
    });
    static READ_SECTION_DUPLICATED: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r#"
            [SECTION]
            1key = "1value"
            | 1col1 | 1col2|
            [SECTION]
            2key = "2value"
            | 2col1 | 2col2|
        "#,
        accepted_sections: &[],
        expected: Some(sections(vec![(
            "SECTION",
            section(
                vec![("2key", string("2value"))],
                vec![row(&["2col1", "2col2"])],
            ),
        )])),
    });
    static READ_FILTER_ROOT_ONLY: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r#"
            nkey = "nvalue"
            | ncol1 | ncol2 |
        "#,
        accepted_sections: &["ACCEPTED"],
        expected: Some(BTreeMap::new()),
    });
    static READ_FILTER_ROOT_THEN_ACCEPTED: LazyLock<ReadTestCase> =
        LazyLock::new(|| ReadTestCase {
            raw: r#"
                nkey = "nvalue"
                | ncol1 | ncol2 |
                [ACCEPTED]
                key = "value"
                | col1 | col2|
            "#,
            accepted_sections: &["ACCEPTED"],
            expected: Some(sections(vec![(
                "ACCEPTED",
                section(vec![("key", string("value"))], vec![row(&["col1", "col2"])]),
            )])),
        });
    static READ_FILTER_ROOT_THEN_FILTERED: LazyLock<ReadTestCase> =
        LazyLock::new(|| ReadTestCase {
            raw: r#"
                nkey = "nvalue"
                | ncol1 | ncol2 |
                [FILTERED]
                key = "value"
                | col1 | col2|
            "#,
            accepted_sections: &["ACCEPTED"],
            expected: Some(BTreeMap::new()),
        });
    static READ_FILTER_ACCEPTED_ONLY: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r#"
            [ACCEPTED]
            key = "value"
            | col1 | col2|
        "#,
        accepted_sections: &["ACCEPTED"],
        expected: Some(sections(vec![(
            "ACCEPTED",
            section(vec![("key", string("value"))], vec![row(&["col1", "col2"])]),
        )])),
    });
    static READ_FILTER_ACCEPTED_THEN_FILTERED: LazyLock<ReadTestCase> =
        LazyLock::new(|| ReadTestCase {
            raw: r#"
                [ACCEPTED]
                key = "value"
                | col1 | col2|
                [FILTERED]
                fkey = "fvalue"
                | fcol1 | fcol2|
            "#,
            accepted_sections: &["ACCEPTED"],
            expected: Some(sections(vec![(
                "ACCEPTED",
                section(vec![("key", string("value"))], vec![row(&["col1", "col2"])]),
            )])),
        });
    static READ_FILTER_DUPLICATED_ACCEPTED_ONLY: LazyLock<ReadTestCase> =
        LazyLock::new(|| ReadTestCase {
            raw: r#"
                [ACCEPTED]
                1key = "1value"
                | 1col1 | 1col2|
                [ACCEPTED]
                2key = "2value"
                | 2col1 | 2col2|
            "#,
            accepted_sections: &["ACCEPTED"],
            expected: Some(sections(vec![(
                "ACCEPTED",
                section(
                    vec![("1key", string("1value"))],
                    vec![row(&["1col1", "1col2"])],
                ),
            )])),
        });
    static READ_FILTER_DUPLICATED_ACCEPTED_WITH_ANOTHER: LazyLock<ReadTestCase> =
        LazyLock::new(|| ReadTestCase {
            raw: r#"
                [ACCEPTED]
                1key = "1value"
                | 1col1 | 1col2|
                [ACCEPTED]
                2key = "2value"
                | 2col1 | 2col2|
            "#,
            accepted_sections: &["ACCEPTED", "ANOTHER"],
            expected: Some(sections(vec![(
                "ACCEPTED",
                section(
                    vec![("1key", string("1value"))],
                    vec![row(&["1col1", "1col2"])],
                ),
            )])),
        });
    static READ_FILTER_FILTERED_ONLY: LazyLock<ReadTestCase> = LazyLock::new(|| ReadTestCase {
        raw: r#"
            [FILTERED]
            key = "value"
            | col1 | col2|
        "#,
        accepted_sections: &["ACCEPTED"],
        expected: Some(BTreeMap::new()),
    });
    static READ_FILTER_FILTERED_THEN_ACCEPTED: LazyLock<ReadTestCase> =
        LazyLock::new(|| ReadTestCase {
            raw: r#"
                [FILTERED]
                fkey = "fvalue"
                | fcol1 | fcol2|
                [ACCEPTED]
                key = "value"
                | col1 | col2|
            "#,
            accepted_sections: &["ACCEPTED"],
            expected: Some(sections(vec![(
                "ACCEPTED",
                section(vec![("key", string("value"))], vec![row(&["col1", "col2"])]),
            )])),
        });

    #[test_case(&*FINISH_STRING_COMPLETE; "complete")]
    #[test_case(&*FINISH_STRING_UNTERMINATED; "unterminated")]
    #[test_case(&*FINISH_STRING_EMPTY; "empty")]
    #[test_case(&*FINISH_STRING_MISSING; "missing")]
    fn finish_string(case: &FinishStringTestCase) {
        let mut parser = Parser::new(case.raw);
        let actual = parser.finish_string().map(|value| match value {
            Value::String(value) => value,
            other => panic!("expected string value, got {other:?}"),
        });
        assert_eq!(case.expected.map(str::to_owned), actual);
    }

    #[test_case(&*FINISH_ARRAY_UNTERMINATED_VALUE; "unterminated value")]
    #[test_case(&*FINISH_ARRAY_MISSING_CLOSE; "missing close")]
    #[test_case(&*FINISH_ARRAY_EMPTY; "empty")]
    #[test_case(&*FINISH_ARRAY_SINGLE_VALUE; "single value")]
    fn finish_array(case: &FinishValueTestCase) {
        let mut parser = Parser::new(case.raw);
        assert_eq!(case.expected, parser.finish_array());
    }

    #[test_case(&*FINISH_DICTIONARY_MISSING_CLOSE; "missing close")]
    #[test_case(&*FINISH_DICTIONARY_MISSING_ASSIGNMENT_VALUE; "missing assignment value")]
    #[test_case(&*FINISH_DICTIONARY_UNTERMINATED_ARRAY; "unterminated array")]
    #[test_case(&*FINISH_DICTIONARY_EMPTY; "empty")]
    #[test_case(&*FINISH_DICTIONARY_WITH_ARRAY; "with array")]
    fn finish_dictionary(case: &FinishValueTestCase) {
        let mut parser = Parser::new(case.raw);
        assert_eq!(case.expected, parser.finish_dictionary());
    }

    #[test_case(&*SLICE_TO_INCLUDING_END; "needle in middle")]
    #[test_case(&*SLICE_TO_INCLUDING_START; "needle at start")]
    fn slice_to_including(case: &SliceTargetTestCase) {
        let mut parser = Parser::new(case.raw);
        assert_eq!(case.expected, parser.slice_to_including(case.target));
        assert_eq!(case.next, parser.cur.next());
    }

    #[test_case(&*SLICE_TO_EXCLUDING_END; "needle in middle")]
    #[test_case(&*SLICE_TO_EXCLUDING_START; "needle at start")]
    #[test_case(&*SLICE_TO_EXCLUDING_ESCAPED; "escaped delimiter")]
    fn slice_to_excluding(case: &SliceTargetTestCase) {
        let mut parser = Parser::new(case.raw);
        assert_eq!(case.expected, parser.slice_to_excluding(case.target));
        assert_eq!(case.next, parser.cur.next());
    }

    #[test_case(&*SLICE_WHILE_UNTIL_MATCH; "progresses until stop")]
    #[test_case(&*SLICE_WHILE_STOPS_IMMEDIATELY; "stops immediately")]
    fn slice_while(case: &SliceWhileTestCase) {
        let mut parser = Parser::new(case.raw);
        assert_eq!(case.expected, parser.slice_while(|c| c != case.stop_at));
        assert_eq!(case.next, parser.cur.next());
    }

    #[test]
    fn parse() {
        let case = &*PARSE_MAIN_CASE;
        let mut parser = Parser::new(case.raw);

        let actual: Vec<_> = parser.by_ref().collect();
        assert_eq!(case.expected, actual);
        assert_eq!(None, parser.next());
    }

    #[test]
    fn display() {
        let case = &*DISPLAY_ARRAY;
        assert_eq!(case.expected, format!("{:#}", case.value));
    }

    #[test_case(&*REPLACE_ESCAPES_PLAIN_TEXT; "plain text")]
    #[test_case(&*REPLACE_ESCAPES_TRAILING_SLASH; "trailing slash")]
    #[test_case(&*REPLACE_ESCAPES_NEWLINE; "newline")]
    #[test_case(&*REPLACE_ESCAPES_TAB; "tab")]
    #[test_case(&*REPLACE_ESCAPES_BACKSLASH; "backslash")]
    #[test_case(&*REPLACE_ESCAPES_LITERAL_SEQUENCE; "literal sequence")]
    #[test_case(&*REPLACE_ESCAPES_PIPE; "pipe")]
    #[test_case(&*REPLACE_ESCAPES_QUOTE_ESCAPED; "quote escaped")]
    #[test_case(&*REPLACE_ESCAPES_QUOTE_LITERAL; "quote literal")]
    #[test_case(&*REPLACE_ESCAPES_UNKNOWN_ESCAPES; "unknown escapes")]
    fn replace_escapes(case: &ReplaceEscapesTestCase) {
        assert_eq!(
            case.expected,
            super::replace_escapes(case.raw, case.escape_quote)
        );
    }

    #[test_case(&*READ_ROOT_STRING; "root string")]
    #[test_case(&*READ_ROOT_ARRAY; "root array")]
    #[test_case(&*READ_ROOT_DICTIONARY; "root dictionary")]
    #[test_case(&*READ_ROOT_MULTILINE_DICTIONARY; "root multiline dictionary")]
    #[test_case(&*READ_ROOT_MISSING_VALUE; "root missing value")]
    #[test_case(&*READ_ROOT_ROWS; "root rows")]
    #[test_case(&*READ_ROOT_ROWS_WITH_EMPTY_CELLS; "root rows with empty cells")]
    #[test_case(&*READ_ROOT_NEGATIVE_NUMBERS; "root negative numbers")]
    #[test_case(&*READ_SECTION_ONCE; "section once")]
    #[test_case(&*READ_SECTION_DUPLICATED; "section duplicated")]
    #[test_case(&*READ_FILTER_ROOT_ONLY; "filter root only")]
    #[test_case(&*READ_FILTER_ROOT_THEN_ACCEPTED; "filter root then accepted")]
    #[test_case(&*READ_FILTER_ROOT_THEN_FILTERED; "filter root then filtered")]
    #[test_case(&*READ_FILTER_ACCEPTED_ONLY; "filter accepted only")]
    #[test_case(&*READ_FILTER_ACCEPTED_THEN_FILTERED; "filter accepted then filtered")]
    #[test_case(&*READ_FILTER_DUPLICATED_ACCEPTED_ONLY; "filter duplicated accepted only")]
    #[test_case(&*READ_FILTER_DUPLICATED_ACCEPTED_WITH_ANOTHER; "filter duplicated accepted with another")]
    #[test_case(&*READ_FILTER_FILTERED_ONLY; "filter filtered only")]
    #[test_case(&*READ_FILTER_FILTERED_THEN_ACCEPTED; "filter filtered then accepted")]
    fn read(case: &ReadTestCase) {
        let actual = if case.accepted_sections.is_empty() {
            Parser::new(case.raw).read()
        } else {
            Parser::new_filtered(case.raw, case.accepted_sections.to_vec()).read()
        };

        assert_eq!(case.expected, actual);
    }
}
