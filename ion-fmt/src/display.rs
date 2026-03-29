//! Formatting display primitives for Ion documents.
//!
//! The formatter is intentionally render-only: it operates on parsed [`Ion`]
//! values and delegates parsing and validation to the `ion` crate.

use crate::columns_width::{Column, ColumnsWidth};
use ion::{Dictionary, Ion, Row, Section, Value};
use std::fmt::{self, Write};
use std::str::FromStr;

/// Formatting options used when rendering Ion documents.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FormatOptions {
    /// Dictionary rendering options.
    pub dictionary: DictionaryOptions,
    /// Section rendering options.
    pub section: SectionOptions,
    /// Document-level rendering options.
    pub document: DocumentOptions,
}

/// Options that control dictionary formatting behavior.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DictionaryOptions {
    /// Style used for dictionary fields.
    pub field: FieldStyle,
}

/// Options that control section-level formatting behavior.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SectionOptions {
    /// Spacing between dictionary fields and table rows in the same section.
    pub spacing: SectionSpacing,
}

/// Options that control document-level formatting behavior.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DocumentOptions {
    /// Spacing at end-of-document.
    pub spacing: DocumentSpacing,
}

/// Formatting style for dictionary string fields.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FieldStyle {
    /// Render dictionary strings as escaped single-line values.
    Singleline,
    /// Preserve embedded newline characters as multiline quoted values.
    #[default]
    Multiline,
}

/// Spacing behavior between dictionary fields and table rows in one section.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SectionSpacing {
    /// Keep the single line break between dictionary fields and table rows.
    NewLine,
    /// Add an extra empty line between dictionary fields and table rows.
    #[default]
    AdditionalNewLine,
}

/// Spacing behavior at end-of-document.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DocumentSpacing {
    /// Keep a single newline at end of document.
    #[default]
    EndNewLine,
    /// Add one extra empty line at end of document.
    AdditionalEndNewLine,
}

impl FromStr for FieldStyle {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "singleline" => Ok(Self::Singleline),
            "multiline" => Ok(Self::Multiline),
            _ => Err(format!(
                "Unsupported `dictionary-field` style `{value}`. Expected `singleline` or `multiline`."
            )),
        }
    }
}

impl FromStr for SectionSpacing {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "newline" => Ok(Self::NewLine),
            "additional-newline" => Ok(Self::AdditionalNewLine),
            _ => Err(format!(
                "Unsupported `section-spacing` style `{value}`. Expected `newline` or `additional-newline`."
            )),
        }
    }
}

impl FromStr for DocumentSpacing {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "end-newline" => Ok(Self::EndNewLine),
            "additional-end-newline" => Ok(Self::AdditionalEndNewLine),
            _ => Err(format!(
                "Unsupported `document-spacing` style `{value}`. Expected `end-newline` or `additional-end-newline`."
            )),
        }
    }
}

/// Display adapter that renders an [`Ion`] document with canonical formatting.
///
/// This is useful when you already have a parsed [`Ion`] value and want a
/// `Display` implementation instead of an owned `String`.
#[derive(Clone, Debug)]
pub struct IonDisplay<'a> {
    ion: &'a Ion,
    options: FormatOptions,
}

impl<'a> IonDisplay<'a> {
    /// Creates a display adapter with explicit formatting options.
    #[must_use]
    pub fn new(ion: &'a Ion, options: FormatOptions) -> Self {
        Self { ion, options }
    }
}

/// Display adapter that renders dictionary fields from a section.
///
/// This is useful when callers want to format only dictionary values instead
/// of a full section or full document.
#[derive(Clone, Debug)]
pub struct DictionaryDisplay<'a> {
    dictionary: &'a Dictionary,
    field: FieldStyle,
}

impl<'a> DictionaryDisplay<'a> {
    /// Creates a dictionary display adapter with explicit dictionary field style.
    #[must_use]
    pub fn new(dictionary: &'a Dictionary, field: FieldStyle) -> Self {
        Self { dictionary, field }
    }
}

/// Display adapter that renders a single dictionary key-value pair.
#[derive(Clone, Debug)]
pub struct DictionaryFieldDisplay<'a> {
    key: &'a str,
    value: &'a Value,
    options: FieldStyle,
}

impl<'a> DictionaryFieldDisplay<'a> {
    /// Creates a dictionary field display adapter with explicit dictionary field style.
    #[must_use]
    pub fn new(key: &'a str, value: &'a Value, field: FieldStyle) -> Self {
        Self {
            key,
            value,
            options: field,
        }
    }
}

#[derive(Clone, Debug)]
struct SectionDisplay<'a> {
    columns_width: ColumnsWidth,
    name: &'a str,
    section: &'a Section,
    options: FormatOptions,
}

#[derive(Clone, Debug)]
struct RowsDisplay<'a> {
    columns_width: &'a ColumnsWidth,
    rows: &'a Vec<Row>,
}

#[derive(Clone, Debug)]
struct RowDisplay<'a> {
    columns_width: &'a ColumnsWidth,
    row: &'a Row,
    row_type: RowTypeDisplay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RowTypeDisplay {
    Header,
    Separator,
    Data,
}

impl<'a> SectionDisplay<'a> {
    fn new(name: &'a str, section: &'a Section, options: FormatOptions) -> Self {
        let columns_width = section
            .rows
            .iter()
            .map(|row| row.iter())
            .collect::<ColumnsWidth>();
        Self {
            columns_width,
            name,
            section,
            options,
        }
    }
}

/// Returns a display adapter rendered with explicit formatting options.
#[must_use]
pub fn display_with_options(ion: &Ion, options: FormatOptions) -> IonDisplay<'_> {
    IonDisplay::new(ion, options)
}

/// Formats an Ion document using explicit formatting options.
#[must_use]
pub fn format_ion_with_options(ion: &Ion, options: FormatOptions) -> String {
    display_with_options(ion, options).to_string()
}

impl fmt::Display for IonDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ion
            .iter()
            .map(|(name, section)| SectionDisplay::new(name, section, self.options))
            .try_for_each(|section| writeln!(f, "{section}"))?;

        if self.ion.iter().next().is_some()
            && self.options.document.spacing == DocumentSpacing::AdditionalEndNewLine
        {
            writeln!(f)?;
        }

        Ok(())
    }
}

impl fmt::Display for SectionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[{}]", self.name)?;
        DictionaryDisplay::new(&self.section.dictionary, self.options.dictionary.field).fmt(f)?;

        if self.section.rows.is_empty() {
            return Ok(());
        }

        if !self.section.dictionary.is_empty()
            && self.options.section.spacing == SectionSpacing::AdditionalNewLine
        {
            writeln!(f)?;
        }

        RowsDisplay {
            columns_width: &self.columns_width,
            rows: &self.section.rows,
        }
        .fmt(f)
    }
}

impl fmt::Display for DictionaryDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dictionary
            .iter()
            .try_for_each(|(key, value)| DictionaryFieldDisplay::new(key, value, self.field).fmt(f))
    }
}

impl fmt::Display for DictionaryFieldDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_dictionary(f, self.value, self.key, self.options, 0)
    }
}

fn display_dictionary(
    f: &mut fmt::Formatter<'_>,
    value: &Value,
    key: &str,
    options: FieldStyle,
    depth: usize,
) -> Result<(), fmt::Error> {
    let current_inden = depth * 4;

    match value {
        Value::Dictionary(dictionary) if options == FieldStyle::Multiline => {
            write!(f, "{0:1$}{2} = ", "", current_inden, key)?;
            write_dictionary_multiline_value_at_depth(f, dictionary, depth, options)?;
        }
        Value::String(text) if options == FieldStyle::Multiline && text.contains('\n') => {
            write_dictionary_multiline_string(f, key, text, depth)?;
        }
        Value::String(_) => {
            write!(f, "{0:1$}{2} = \"{3}\"", "", current_inden, key, value)?;
        }
        _ => {
            write!(f, "{0:1$}{2} = {3}", "", current_inden, key, value)?;
        }
    }

    writeln!(f)
}

impl fmt::Display for RowsDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.rows.is_empty() {
            return Ok(());
        }

        self.rows
            .iter()
            .enumerate()
            .try_for_each(|(row_idx, row)| {
                RowDisplay {
                    columns_width: self.columns_width,
                    row,
                    row_type: self.columns_width.row_type(row_idx),
                }
                .fmt(f)
            })?;

        Ok(())
    }
}

impl fmt::Display for RowDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.row_type {
            RowTypeDisplay::Header => header(f, self),
            RowTypeDisplay::Separator => header_separator(f, self),
            RowTypeDisplay::Data => write_data(f, self),
        }
    }
}

fn write_dictionary_multiline_string(
    f: &mut fmt::Formatter<'_>,
    key: &str,
    text: &str,
    depth: usize,
) -> fmt::Result {
    write!(f, "{0:1$}{2} = \"", "", depth * 4, key)?;

    for ch in text.chars() {
        match ch {
            '\\' => f.write_str("\\\\")?,
            '"' => f.write_str("\\\"")?,
            _ => f.write_char(ch)?,
        }
    }
    write!(f, "\"")
}

fn write_dictionary_multiline_value_at_depth(
    f: &mut fmt::Formatter<'_>,
    dictionary: &Dictionary,
    depth: usize,
    options: FieldStyle,
) -> fmt::Result {
    writeln!(f, "{{")?;

    for (key, value) in dictionary {
        display_dictionary(f, value, key, options, depth + 1)?;
    }

    write!(f, "{0:1$}}}", "", depth * 4)
}

fn header(f: &mut fmt::Formatter<'_>, columns: &RowDisplay<'_>) -> fmt::Result {
    let mut columns_iter = columns.row.iter().enumerate();

    if let Some((idx, ion_value)) = columns_iter.next() {
        let column = columns.columns_width.column(idx);
        let text = format!("{ion_value}");
        let (left_alignment, right_alignment) = center_header_column_alignment(column, &text);

        write!(
            f,
            "| {} |",
            format_args!("{0:1$}{2}{0:3$}", "", left_alignment, text, right_alignment)
        )?;

        for (idx, ion_value) in columns_iter {
            let column = columns.columns_width.column(idx);
            let text = format!("{ion_value}");
            let (left_alignment, right_alignment) = center_header_column_alignment(column, &text);

            write!(
                f,
                " {} |",
                format_args!(
                    "{0:^1$}{2}{0:3$}",
                    "", left_alignment, text, right_alignment
                )
            )?;
        }

        writeln!(f)?;
    }

    Ok(())
}

fn center_header_column_alignment(column: Column, text: &str) -> (usize, usize) {
    let spaces_width = column.width.checked_sub(text.len()).unwrap_or(text.len());
    let even_width = spaces_width / 2;
    let (modulo_left, modulo_right) = if text.len() & 0x1 == 0x1 {
        (0, spaces_width % 2)
    } else {
        (spaces_width % 2, 0)
    };

    (even_width + modulo_left, even_width + modulo_right)
}

fn header_separator(f: &mut fmt::Formatter<'_>, columns: &RowDisplay<'_>) -> fmt::Result {
    let mut header_iter = columns.row.iter().enumerate().map(|(index, _)| index);

    if let Some(idx) = header_iter.next() {
        write!(f, "|{1:-^0$}|", columns.columns_width.width(idx) + 2, "")?;

        for idx in header_iter {
            write!(f, "{1:-^0$}|", columns.columns_width.width(idx) + 2, "")?;
        }

        writeln!(f)?;
    }

    Ok(())
}

fn write_data(f: &mut fmt::Formatter<'_>, columns: &RowDisplay<'_>) -> fmt::Result {
    let mut values_iter = columns.row.iter().enumerate();

    if let Some((idx, ion_value)) = values_iter.next() {
        let column = columns.columns_width.column(idx);
        write_data_first_column(f, column, ion_value)?;

        for (idx, ion_value) in values_iter {
            let column = columns.columns_width.column(idx);
            write_data_next_columns(f, column, ion_value)?;
        }

        writeln!(f)?;
    }

    Ok(())
}

fn write_data_first_column(
    f: &mut fmt::Formatter<'_>,
    column: Column,
    ion_value: &Value,
) -> fmt::Result {
    if column.is_number() {
        write!(f, "| {1:>0$} |", column.width, format!("{ion_value}"))?;
    } else {
        write!(f, "| {1:<0$} |", column.width, format!("{ion_value}"))?;
    }

    Ok(())
}

fn write_data_next_columns(
    f: &mut fmt::Formatter<'_>,
    column: Column,
    ion_value: &Value,
) -> fmt::Result {
    if column.is_number() {
        write!(f, " {1:>0$} |", column.width, format!("{ion_value}"))?;
    } else {
        write!(f, " {1:<0$} |", column.width, format!("{ion_value}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::columns_width::{Column, ColumnType};
    use indoc::indoc;
    use ion::Dictionary;
    use pretty_assertions::assert_eq;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct IonFormatTestCase {
        description: &'static str,
        raw: &'static str,
        options: FormatOptions,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct DictionaryDisplayTestCase {
        description: &'static str,
        dictionary: Dictionary,
        field: FieldStyle,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct DictionaryFieldDisplayTestCase {
        description: &'static str,
        key: &'static str,
        value: Value,
        field: FieldStyle,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct WriteDictionaryMultilineStringTestCase {
        description: &'static str,
        key: &'static str,
        text: &'static str,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct RowDisplayTestCase {
        description: &'static str,
        row: Row,
        row_type: RowTypeDisplay,
        columns_width: ColumnsWidth,
        expected: &'static str,
    }

    fn string(value: &str) -> Value {
        Value::String(value.into())
    }

    fn dictionary(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Dictionary {
        let mut dictionary = Dictionary::new();
        for (key, value) in entries {
            dictionary.insert(key.into(), value);
        }
        dictionary
    }

    #[test]
    fn section_spacing_default_is_additional_newline() {
        assert_eq!(SectionSpacing::AdditionalNewLine, SectionSpacing::default());
        assert_eq!(
            SectionSpacing::AdditionalNewLine,
            FormatOptions::default().section.spacing
        );
    }

    #[test]
    fn document_spacing_default_is_newline() {
        assert_eq!(DocumentSpacing::EndNewLine, DocumentSpacing::default());
        assert_eq!(
            DocumentSpacing::EndNewLine,
            FormatOptions::default().document.spacing
        );
    }

    static ION_FORMAT_CASE: LazyLock<IonFormatTestCase> = LazyLock::new(|| IonFormatTestCase {
        description: "formats ion document",
        raw: indoc! {r#"
            [ALPHA]
            name = "foo"

            [BETA]
            | num | text |
            |-----|------|
            | 1   | A    |
            | 22  | B    |
        "#},
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Singleline,
            },
            section: SectionOptions {
                spacing: SectionSpacing::AdditionalNewLine,
            },
            document: DocumentOptions {
                spacing: DocumentSpacing::EndNewLine,
            },
        },
        expected: indoc! {r#"
            [ALPHA]
            name = "foo"

            [BETA]
            | num | text |
            |-----|------|
            |   1 | A    |
            |  22 | B    |

        "#},
    });
    static ION_FORMAT_NON_STRING_DICTIONARY_CASE: LazyLock<IonFormatTestCase> =
        LazyLock::new(|| IonFormatTestCase {
            description: "formats non string dictionary value",
            raw: indoc! {r"
                [ALPHA]
                value = 7
            "},
            options: FormatOptions {
                dictionary: DictionaryOptions {
                    field: FieldStyle::Singleline,
                },
                section: SectionOptions {
                    spacing: SectionSpacing::AdditionalNewLine,
                },
                document: DocumentOptions {
                    spacing: DocumentSpacing::EndNewLine,
                },
            },
            expected: indoc! {r"
                [ALPHA]
                value = 7

            "},
        });
    const TEST_1A_BASE_CASE: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_1A base input with singleline dictionary, single newline section, single newline document",
        raw: indoc! {r#"
            [Data]
            select = "
                SELECT column
                FROM table t1
                INNER JOIN t2
                    ON t1.id = t2.
                WHERE t1.userid = {{ user_id }}
                ORDER BY name ASC
            "
            |   col1   |
            |--------|
            | name1 |
            | name2 |
        "#},
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Singleline,
            },
            section: SectionOptions {
                spacing: SectionSpacing::NewLine,
            },
            document: DocumentOptions {
                spacing: DocumentSpacing::EndNewLine,
            },
        },
        expected: indoc! {r#"
            [Data]
            select = "\n    SELECT column\n    FROM table t1\n    INNER JOIN t2\n        ON t1.id = t2.\n    WHERE t1.userid = {{ user_id }}\n    ORDER BY name ASC\n"
            |  col1 |
            |-------|
            | name1 |
            | name2 |

        "#},
    };
    const TEST_1B_DICTIONARY_MULTILINE_CASE: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_1B same input as TEST_1A with multiline dictionary field style",
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Multiline,
            },
            ..TEST_1A_BASE_CASE.options
        },
        expected: indoc! {r#"
            [Data]
            select = "
                SELECT column
                FROM table t1
                INNER JOIN t2
                    ON t1.id = t2.
                WHERE t1.userid = {{ user_id }}
                ORDER BY name ASC
            "
            |  col1 |
            |-------|
            | name1 |
            | name2 |

        "#},
        ..TEST_1A_BASE_CASE
    };
    const TEST_1C_SECTION_ADDITIONAL_NEWLINE_CASE: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_1C same input as TEST_1A with additional newline between dictionary and table",
        options: FormatOptions {
            section: SectionOptions {
                spacing: SectionSpacing::AdditionalNewLine,
            },
            ..TEST_1A_BASE_CASE.options
        },
        expected: indoc! {r#"
                [Data]
                select = "\n    SELECT column\n    FROM table t1\n    INNER JOIN t2\n        ON t1.id = t2.\n    WHERE t1.userid = {{ user_id }}\n    ORDER BY name ASC\n"

                |  col1 |
                |-------|
                | name1 |
                | name2 |

            "#},
        ..TEST_1A_BASE_CASE
    };
    const TEST_1D_DOCUMENT_ADDITIONAL_NEWLINE_CASE: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_1D same input as TEST_1A with additional newline at document end",
        options: FormatOptions {
            document: DocumentOptions {
                spacing: DocumentSpacing::AdditionalEndNewLine,
            },
            ..TEST_1A_BASE_CASE.options
        },
        expected: indoc! {r#"
                [Data]
                select = "\n    SELECT column\n    FROM table t1\n    INNER JOIN t2\n        ON t1.id = t2.\n    WHERE t1.userid = {{ user_id }}\n    ORDER BY name ASC\n"
                |  col1 |
                |-------|
                | name1 |
                | name2 |


            "#},
        ..TEST_1A_BASE_CASE
    };
    const TEST_1E_DEFAULT_BEHAVIOR_CASE: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_1E same input as TEST_1A with current default option combination",
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Multiline,
            },
            section: SectionOptions {
                spacing: SectionSpacing::AdditionalNewLine,
            },
            document: DocumentOptions {
                spacing: DocumentSpacing::EndNewLine,
            },
        },
        expected: indoc! {r#"
            [Data]
            select = "
                SELECT column
                FROM table t1
                INNER JOIN t2
                    ON t1.id = t2.
                WHERE t1.userid = {{ user_id }}
                ORDER BY name ASC
            "

            |  col1 |
            |-------|
            | name1 |
            | name2 |

        "#},
        ..TEST_1A_BASE_CASE
    };
    const TEST_2_WITHOUT_HEADER_TABLE: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_2 table rows without explicit header separator are formatted consistently",
        raw: indoc! {r"
            [WITHOUT_HEADER]
            | 1 | alpha | PL | 11.2 | ok |
            | 2 | beta | DE | 9 | hold |
            | 3 | gamma | UK | 13.75 | ok |
        "},
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Singleline,
            },
            section: SectionOptions {
                spacing: SectionSpacing::AdditionalNewLine,
            },
            document: DocumentOptions {
                spacing: DocumentSpacing::EndNewLine,
            },
        },
        expected: indoc! {r"
            [WITHOUT_HEADER]
            | 1 | alpha | PL |  11.2 | ok   |
            | 2 | beta  | DE |     9 | hold |
            | 3 | gamma | UK | 13.75 | ok   |

        "},
    };
    const TEST_3A_NESTED_DICTIONARY: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_3A formats dictionary with string, array, and nested dictionary values",
        raw: indoc! {r#"
            [CONTRACT]
            country = "Poland"
            markets = ["PL", "DE", "UK"]
            75042 = {
                view = "SV"
                loc  = ["M", "B"]
                dist = { beach_km = 4.1 }
            }
        "#},
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Singleline,
            },
            section: SectionOptions {
                spacing: SectionSpacing::AdditionalNewLine,
            },
            document: DocumentOptions {
                spacing: DocumentSpacing::EndNewLine,
            },
        },
        #[cfg(not(feature = "dictionary-indexmap"))]
        expected: indoc! {r#"
            [CONTRACT]
            75042 = { dist = { beach_km = 4.1 }, loc = [ "M", "B" ], view = "SV" }
            country = "Poland"
            markets = [ "PL", "DE", "UK" ]

        "#},
        #[cfg(feature = "dictionary-indexmap")]
        expected: indoc! {r#"
            [CONTRACT]
            country = "Poland"
            markets = [ "PL", "DE", "UK" ]
            75042 = { view = "SV", loc = [ "M", "B" ], dist = { beach_km = 4.1 } }

        "#},
    };
    const TEST_3B_NESTED_DICTIONARY: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_3B same input as TEST_3A with multiline dictionary field style",
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Multiline,
            },
            ..TEST_3A_NESTED_DICTIONARY.options
        },
        #[cfg(not(feature = "dictionary-indexmap"))]
        expected: indoc! {r#"
            [CONTRACT]
            75042 = {
                dist = {
                    beach_km = 4.1
                }
                loc = [ "M", "B" ]
                view = "SV"
            }
            country = "Poland"
            markets = [ "PL", "DE", "UK" ]

        "#},
        #[cfg(feature = "dictionary-indexmap")]
        expected: indoc! {r#"
            [CONTRACT]
            country = "Poland"
            markets = [ "PL", "DE", "UK" ]
            75042 = {
                view = "SV"
                loc = [ "M", "B" ]
                dist = {
                    beach_km = 4.1
                }
            }

        "#},
        ..TEST_3A_NESTED_DICTIONARY
    };
    const TEST_3C_NESTED_DICTIONARY: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_3C same input as TEST_3A with default dictionary field style",
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Multiline,
            },
            document: DocumentOptions {
                spacing: DocumentSpacing::AdditionalEndNewLine,
            },
            ..TEST_3A_NESTED_DICTIONARY.options
        },
        #[cfg(not(feature = "dictionary-indexmap"))]
        expected: indoc! {r#"
            [CONTRACT]
            75042 = {
                dist = {
                    beach_km = 4.1
                }
                loc = [ "M", "B" ]
                view = "SV"
            }
            country = "Poland"
            markets = [ "PL", "DE", "UK" ]


        "#},
        #[cfg(feature = "dictionary-indexmap")]
        expected: indoc! {r#"
            [CONTRACT]
            country = "Poland"
            markets = [ "PL", "DE", "UK" ]
            75042 = {
                view = "SV"
                loc = [ "M", "B" ]
                dist = {
                    beach_km = 4.1
                }
            }


        "#},
        ..TEST_3A_NESTED_DICTIONARY
    };
    const TEST_4A_DEEP_NESTED_DICTIONARY: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_4A formats deep nested dictionary field in singleline mode",
        raw: indoc! {r#"
            [CONFIG]
            config = {
                zeta = {
                    release = {
                        minor = 2
                        major = 1
                    }
                    flags = [true, false]
                    select = "
                        SELECT column
                        FROM table t4
                        INNER JOIN t6
                            ON t4.id = t6.id
                        WHERE t4.userid = {{ user_id }}
                        ORDER BY name ASC
                    "
                }
                alpha = "pkg"
                select = "
                    SELECT column
                    FROM table t1
                    INNER JOIN t2
                        ON t1.id = t2.something
                    WHERE t1.userid = {{ user_id }}
                    ORDER BY name ASC
                "
            }
        "#},
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Singleline,
            },
            section: SectionOptions {
                spacing: SectionSpacing::AdditionalNewLine,
            },
            document: DocumentOptions {
                spacing: DocumentSpacing::EndNewLine,
            },
        },
        #[cfg(not(feature = "dictionary-indexmap"))]
        expected: indoc! {r#"
            [CONFIG]
            config = { alpha = "pkg", select = "\n        SELECT column\n        FROM table t1\n        INNER JOIN t2\n            ON t1.id = t2.something\n        WHERE t1.userid = {{ user_id }}\n        ORDER BY name ASC\n    ", zeta = { flags = [ true, false ], release = { major = 1, minor = 2 }, select = "\n            SELECT column\n            FROM table t4\n            INNER JOIN t6\n                ON t4.id = t6.id\n            WHERE t4.userid = {{ user_id }}\n            ORDER BY name ASC\n        " } }

        "#},
        #[cfg(feature = "dictionary-indexmap")]
        expected: indoc! {r#"
            [CONFIG]
            config = { zeta = { release = { minor = 2, major = 1 }, flags = [ true, false ], select = "\n            SELECT column\n            FROM table t4\n            INNER JOIN t6\n                ON t4.id = t6.id\n            WHERE t4.userid = {{ user_id }}\n            ORDER BY name ASC\n        " }, alpha = "pkg", select = "\n        SELECT column\n        FROM table t1\n        INNER JOIN t2\n            ON t1.id = t2.something\n        WHERE t1.userid = {{ user_id }}\n        ORDER BY name ASC\n    " }

        "#},
    };
    const TEST_4B_DEEP_NESTED_DICTIONARY: IonFormatTestCase = IonFormatTestCase {
        description: "TEST_4B same input as TEST_4A in multiline mode",
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Multiline,
            },
            section: SectionOptions {
                spacing: SectionSpacing::AdditionalNewLine,
            },
            document: DocumentOptions {
                spacing: DocumentSpacing::EndNewLine,
            },
        },
        #[cfg(not(feature = "dictionary-indexmap"))]
        expected: indoc! {r#"
            [CONFIG]
            config = {
                alpha = "pkg"
                select = "
                    SELECT column
                    FROM table t1
                    INNER JOIN t2
                        ON t1.id = t2.something
                    WHERE t1.userid = {{ user_id }}
                    ORDER BY name ASC
                "
                zeta = {
                    flags = [ true, false ]
                    release = {
                        major = 1
                        minor = 2
                    }
                    select = "
                        SELECT column
                        FROM table t4
                        INNER JOIN t6
                            ON t4.id = t6.id
                        WHERE t4.userid = {{ user_id }}
                        ORDER BY name ASC
                    "
                }
            }

        "#},
        #[cfg(feature = "dictionary-indexmap")]
        expected: indoc! {r#"
            [CONFIG]
            config = {
                zeta = {
                    release = {
                        minor = 2
                        major = 1
                    }
                    flags = [ true, false ]
                    select = "
                        SELECT column
                        FROM table t4
                        INNER JOIN t6
                            ON t4.id = t6.id
                        WHERE t4.userid = {{ user_id }}
                        ORDER BY name ASC
                    "
                }
                alpha = "pkg"
                select = "
                    SELECT column
                    FROM table t1
                    INNER JOIN t2
                        ON t1.id = t2.something
                    WHERE t1.userid = {{ user_id }}
                    ORDER BY name ASC
                "
            }

        "#},
        ..TEST_4A_DEEP_NESTED_DICTIONARY
    };
    const TEST_5A_TABLE_ONLY_WITH_DEFAULT_SECTION_SPACING_CASE: IonFormatTestCase =
        IonFormatTestCase {
            description: "TEST_5A table-only input does not add section spacing when dictionary is empty",
            raw: indoc! {r"
            [TABLE]
            | c |
            |---|
            | 1 |
        "},
            options: FormatOptions {
                dictionary: DictionaryOptions {
                    field: FieldStyle::Singleline,
                },
                section: SectionOptions {
                    spacing: SectionSpacing::AdditionalNewLine,
                },
                document: DocumentOptions {
                    spacing: DocumentSpacing::EndNewLine,
                },
            },
            expected: indoc! {r"
            [TABLE]
            | c |
            |---|
            | 1 |

        "},
        };
    #[test_case(&*ION_FORMAT_CASE)]
    #[test_case(&*ION_FORMAT_NON_STRING_DICTIONARY_CASE)]
    #[test_case(&TEST_1A_BASE_CASE)]
    #[test_case(&TEST_1B_DICTIONARY_MULTILINE_CASE)]
    #[test_case(&TEST_1C_SECTION_ADDITIONAL_NEWLINE_CASE)]
    #[test_case(&TEST_1D_DOCUMENT_ADDITIONAL_NEWLINE_CASE)]
    #[test_case(&TEST_1E_DEFAULT_BEHAVIOR_CASE)]
    #[test_case(&TEST_2_WITHOUT_HEADER_TABLE)]
    #[test_case(&TEST_3A_NESTED_DICTIONARY)]
    #[test_case(&TEST_3B_NESTED_DICTIONARY)]
    #[test_case(&TEST_3C_NESTED_DICTIONARY)]
    #[test_case(&TEST_4A_DEEP_NESTED_DICTIONARY)]
    #[test_case(&TEST_4B_DEEP_NESTED_DICTIONARY)]
    #[test_case(&TEST_5A_TABLE_ONLY_WITH_DEFAULT_SECTION_SPACING_CASE)]
    fn format_ion_document(case: &IonFormatTestCase) {
        let ion = case.raw.parse::<Ion>().unwrap();
        assert_eq!(
            case.expected,
            format_ion_with_options(&ion, case.options),
            "{}",
            case.description
        );
        assert_eq!(
            case.expected,
            display_with_options(&ion, case.options).to_string(),
            "{}",
            case.description
        );
    }

    static DICTIONARY_DISPLAY_SINGLELINE_CASE: LazyLock<DictionaryDisplayTestCase> =
        LazyLock::new(|| DictionaryDisplayTestCase {
            description: "dictionary display with singleline style",
            dictionary: dictionary([("a", string("foo")), ("query", string("\nSELECT 1\n"))]),
            field: FieldStyle::Singleline,
            expected: "a = \"foo\"\nquery = \"\\nSELECT 1\\n\"\n",
        });
    static DICTIONARY_DISPLAY_MULTILINE_CASE: LazyLock<DictionaryDisplayTestCase> =
        LazyLock::new(|| DictionaryDisplayTestCase {
            description: "dictionary display with multiline style",
            dictionary: dictionary([("a", string("foo")), ("query", string("\nSELECT 1\n"))]),
            field: FieldStyle::Multiline,
            expected: "a = \"foo\"\nquery = \"\nSELECT 1\n\"\n",
        });

    #[test_case(&*DICTIONARY_DISPLAY_SINGLELINE_CASE)]
    #[test_case(&*DICTIONARY_DISPLAY_MULTILINE_CASE)]
    fn display_dictionary(case: &DictionaryDisplayTestCase) {
        assert_eq!(
            case.expected,
            DictionaryDisplay::new(&case.dictionary, case.field).to_string(),
            "{}",
            case.description
        );
    }

    static DICTIONARY_FIELD_NON_STRING_CASE: LazyLock<DictionaryFieldDisplayTestCase> =
        LazyLock::new(|| DictionaryFieldDisplayTestCase {
            description: "dictionary field non string value",
            key: "count",
            value: Value::Integer(7),
            field: FieldStyle::Singleline,
            expected: "count = 7\n",
        });
    static DICTIONARY_FIELD_MULTILINE_SINGLELINE_CASE: LazyLock<DictionaryFieldDisplayTestCase> =
        LazyLock::new(|| DictionaryFieldDisplayTestCase {
            description: "dictionary field multiline value with singleline style",
            key: "query",
            value: string("\nSELECT 1\n"),
            field: FieldStyle::Singleline,
            expected: "query = \"\\nSELECT 1\\n\"\n",
        });
    static DICTIONARY_FIELD_MULTILINE_MULTILINE_CASE: LazyLock<DictionaryFieldDisplayTestCase> =
        LazyLock::new(|| DictionaryFieldDisplayTestCase {
            description: "dictionary field multiline value with multiline style",
            key: "query",
            value: string("\nSELECT 1\n"),
            field: FieldStyle::Multiline,
            expected: "query = \"\nSELECT 1\n\"\n",
        });
    static DICTIONARY_FIELD_DICTIONARY_MULTILINE_CASE: LazyLock<DictionaryFieldDisplayTestCase> =
        LazyLock::new(|| {
            let value = Value::Dictionary(dictionary([
                ("view", string("SV")),
                ("loc", Value::Array(vec![string("M"), string("B")])),
                (
                    "dist",
                    Value::Dictionary(dictionary([("beach_km", Value::Float(4.1))])),
                ),
            ]));
            DictionaryFieldDisplayTestCase {
                description: "dictionary field dictionary value with multiline style",
                key: "75042",
                value,
                field: FieldStyle::Multiline,
                expected: if cfg!(feature = "dictionary-indexmap") {
                    indoc! {r#"
                        75042 = {
                            view = "SV"
                            loc = [ "M", "B" ]
                            dist = {
                                beach_km = 4.1
                            }
                        }
                    "#}
                } else {
                    indoc! {r#"
                        75042 = {
                            dist = {
                                beach_km = 4.1
                            }
                            loc = [ "M", "B" ]
                            view = "SV"
                        }
                    "#}
                },
            }
        });
    #[test_case(&*DICTIONARY_FIELD_NON_STRING_CASE)]
    #[test_case(&*DICTIONARY_FIELD_MULTILINE_SINGLELINE_CASE)]
    #[test_case(&*DICTIONARY_FIELD_MULTILINE_MULTILINE_CASE)]
    #[test_case(&*DICTIONARY_FIELD_DICTIONARY_MULTILINE_CASE)]
    fn display_dictionary_field(case: &DictionaryFieldDisplayTestCase) {
        assert_eq!(
            case.expected,
            DictionaryFieldDisplay::new(case.key, &case.value, case.field).to_string(),
            "{}",
            case.description
        );
    }

    fn render_dictionary_multiline_string(key: &str, text: &str) -> String {
        struct DictionaryMultilineStringDisplay<'a> {
            key: &'a str,
            text: &'a str,
        }

        impl std::fmt::Display for DictionaryMultilineStringDisplay<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write_dictionary_multiline_string(f, self.key, self.text, 0)
            }
        }

        DictionaryMultilineStringDisplay { key, text }.to_string()
    }

    const WRITE_MULTILINE_DICTIONARY_STRING_EMPTY_CASE: WriteDictionaryMultilineStringTestCase =
        WriteDictionaryMultilineStringTestCase {
            description: "writes empty multiline dictionary string",
            key: "query",
            text: "",
            expected: "query = \"\"",
        };
    const WRITE_MULTILINE_DICTIONARY_STRING_MULTILINE_CASE: WriteDictionaryMultilineStringTestCase =
        WriteDictionaryMultilineStringTestCase {
            description: "preserves multiline text and trailing newline",
            key: "query",
            text: "\nSELECT 1\nFROM dual\n",
            expected: "query = \"\nSELECT 1\nFROM dual\n\"",
        };
    const WRITE_MULTILINE_DICTIONARY_STRING_ESCAPES_QUOTE_CASE:
        WriteDictionaryMultilineStringTestCase = WriteDictionaryMultilineStringTestCase {
        description: "escapes quotes in multiline dictionary string",
        key: "query",
        text: "value \"quoted\"",
        expected: "query = \"value \\\"quoted\\\"\"",
    };
    const WRITE_MULTILINE_DICTIONARY_STRING_ESCAPES_BACKSLASH_CASE:
        WriteDictionaryMultilineStringTestCase = WriteDictionaryMultilineStringTestCase {
        description: "escapes backslashes in multiline dictionary string",
        key: "query",
        text: r"C:\work\ion",
        expected: "query = \"C:\\\\work\\\\ion\"",
    };
    const WRITE_MULTILINE_DICTIONARY_STRING_ESCAPES_QUOTE_AND_BACKSLASH_CASE:
        WriteDictionaryMultilineStringTestCase = WriteDictionaryMultilineStringTestCase {
        description: "escapes quotes and backslashes while preserving newlines",
        key: "query",
        text: "path=\"C:\\work\"\n-- done",
        expected: "query = \"path=\\\"C:\\\\work\\\"\n-- done\"",
    };

    #[test_case(&WRITE_MULTILINE_DICTIONARY_STRING_EMPTY_CASE)]
    #[test_case(&WRITE_MULTILINE_DICTIONARY_STRING_MULTILINE_CASE)]
    #[test_case(&WRITE_MULTILINE_DICTIONARY_STRING_ESCAPES_QUOTE_CASE)]
    #[test_case(&WRITE_MULTILINE_DICTIONARY_STRING_ESCAPES_BACKSLASH_CASE)]
    #[test_case(&WRITE_MULTILINE_DICTIONARY_STRING_ESCAPES_QUOTE_AND_BACKSLASH_CASE)]
    fn write_dictionary_multiline_string_cases(case: &WriteDictionaryMultilineStringTestCase) {
        assert_eq!(
            case.expected,
            render_dictionary_multiline_string(case.key, case.text),
            "{}",
            case.description
        );
    }

    static HEADER_ROW_DISPLAY_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            description: "header row",
            row: vec![string("num"), string("total")],
            row_type: RowTypeDisplay::Header,
            columns_width: ColumnsWidth::new(vec![
                Column {
                    width: 3,
                    typ: ColumnType::Text,
                },
                Column {
                    width: 5,
                    typ: ColumnType::Text,
                },
            ]),
            expected: "| num | total |\n",
        });
    static SEPARATOR_ROW_DISPLAY_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            description: "separator row",
            row: vec![string("num"), string("total")],
            row_type: RowTypeDisplay::Separator,
            columns_width: ColumnsWidth::new(vec![
                Column {
                    width: 3,
                    typ: ColumnType::Text,
                },
                Column {
                    width: 5,
                    typ: ColumnType::Text,
                },
            ]),
            expected: "|-----|-------|\n",
        });
    static DATA_ROW_DISPLAY_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            description: "data row",
            row: vec![Value::Integer(12), string("A")],
            row_type: RowTypeDisplay::Data,
            columns_width: ColumnsWidth::new(vec![
                Column {
                    width: 3,
                    typ: ColumnType::Number,
                },
                Column {
                    width: 5,
                    typ: ColumnType::Text,
                },
            ]),
            expected: "|  12 | A     |\n",
        });
    static HEADER_ROW_DISPLAY_EVEN_TEXT_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            description: "header row with even text",
            row: vec![string("ABCD"), string("EF")],
            row_type: RowTypeDisplay::Header,
            columns_width: ColumnsWidth::new(vec![
                Column {
                    width: 5,
                    typ: ColumnType::Text,
                },
                Column {
                    width: 4,
                    typ: ColumnType::Text,
                },
            ]),
            expected: "|  ABCD |  EF  |\n",
        });
    static DATA_ROW_DISPLAY_TEXT_THEN_NUMBER_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            description: "data row with text first and number next",
            row: vec![string("A"), Value::Integer(7)],
            row_type: RowTypeDisplay::Data,
            columns_width: ColumnsWidth::new(vec![
                Column {
                    width: 3,
                    typ: ColumnType::Text,
                },
                Column {
                    width: 4,
                    typ: ColumnType::Number,
                },
            ]),
            expected: "| A   |    7 |\n",
        });
    static EMPTY_HEADER_ROW_DISPLAY_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            description: "empty header row",
            row: vec![],
            row_type: RowTypeDisplay::Header,
            columns_width: ColumnsWidth::new(vec![]),
            expected: "",
        });
    static EMPTY_SEPARATOR_ROW_DISPLAY_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            description: "empty separator row",
            row: vec![],
            row_type: RowTypeDisplay::Separator,
            columns_width: ColumnsWidth::new(vec![]),
            expected: "",
        });
    static EMPTY_DATA_ROW_DISPLAY_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            description: "empty data row",
            row: vec![],
            row_type: RowTypeDisplay::Data,
            columns_width: ColumnsWidth::new(vec![]),
            expected: "",
        });

    #[test_case(&*HEADER_ROW_DISPLAY_CASE)]
    #[test_case(&*SEPARATOR_ROW_DISPLAY_CASE)]
    #[test_case(&*DATA_ROW_DISPLAY_CASE)]
    #[test_case(&*HEADER_ROW_DISPLAY_EVEN_TEXT_CASE)]
    #[test_case(&*DATA_ROW_DISPLAY_TEXT_THEN_NUMBER_CASE)]
    #[test_case(&*EMPTY_HEADER_ROW_DISPLAY_CASE)]
    #[test_case(&*EMPTY_SEPARATOR_ROW_DISPLAY_CASE)]
    #[test_case(&*EMPTY_DATA_ROW_DISPLAY_CASE)]
    fn display_row(case: &RowDisplayTestCase) {
        let row_display = RowDisplay {
            columns_width: &case.columns_width,
            row: &case.row,
            row_type: case.row_type,
        };
        assert_eq!(
            case.expected,
            row_display.to_string(),
            "{}",
            case.description
        );
    }
}
