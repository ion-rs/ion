//! Formatting display primitives for Ion documents.
//!
//! The formatter is intentionally render-only: it operates on parsed [`Ion`]
//! values and delegates parsing and validation to the `ion` crate.

use crate::columns_width::{Column, ColumnsWidth};
use ion::{Ion, Row, Section, Value};
use std::fmt;

/// Display adapter that renders an [`Ion`] document with canonical formatting.
#[derive(Clone, Debug)]
pub struct IonDisplay<'a> {
    ion: &'a Ion,
}

impl<'a> IonDisplay<'a> {
    /// Creates a display adapter for an Ion document.
    #[must_use]
    pub fn new(ion: &'a Ion) -> Self {
        Self { ion }
    }
}

#[derive(Clone, Debug)]
struct SectionDisplay<'a> {
    columns_width: ColumnsWidth,
    name: &'a str,
    section: &'a Section,
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
    fn new(name: &'a str, section: &'a Section) -> Self {
        let columns_width = section
            .rows
            .iter()
            .map(|row| row.iter())
            .collect::<ColumnsWidth>();
        Self {
            columns_width,
            name,
            section,
        }
    }
}

/// Returns a display adapter that can be rendered with `to_string()`.
#[must_use]
pub fn display(ion: &Ion) -> IonDisplay<'_> {
    IonDisplay::new(ion)
}

/// Formats an Ion document into its canonical string representation.
#[must_use]
pub fn format_ion(ion: &Ion) -> String {
    display(ion).to_string()
}

impl fmt::Display for IonDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ion
            .iter()
            .map(|(name, section)| SectionDisplay::new(name, section))
            .try_for_each(|section| writeln!(f, "{section}"))
    }
}

impl fmt::Display for SectionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[{}]", self.name)?;
        dictionary(f, self.section.dictionary.iter())?;

        if self.section.rows.is_empty() {
            return Ok(());
        }

        RowsDisplay {
            columns_width: &self.columns_width,
            rows: &self.section.rows,
        }
        .fmt(f)
    }
}

impl fmt::Display for RowsDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rows_iter = self.rows.iter();

        if let Some(row) = rows_iter.next() {
            RowDisplay {
                columns_width: self.columns_width,
                row,
                row_type: RowTypeDisplay::Header,
            }
            .fmt(f)?;

            RowDisplay {
                columns_width: self.columns_width,
                row,
                row_type: RowTypeDisplay::Separator,
            }
            .fmt(f)?;

            rows_iter.skip(1).try_for_each(|row| {
                RowDisplay {
                    columns_width: self.columns_width,
                    row,
                    row_type: data_or_separator(row.iter()),
                }
                .fmt(f)
            })?;
        }

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

fn dictionary<'a>(
    f: &mut fmt::Formatter<'_>,
    mut fields: impl Iterator<Item = (&'a String, &'a Value)>,
) -> fmt::Result {
    fields.try_for_each(|(key, value)| {
        if value.is_string() {
            writeln!(f, "{key} = \"{value}\"")
        } else {
            writeln!(f, "{key} = {value}")
        }
    })
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

/// Detects whether a row represents table data or a separator row.
///
/// Separator rows are string-only rows containing spaces and `-` characters.
pub(crate) fn data_or_separator<'a>(mut row: impl Iterator<Item = &'a Value>) -> RowTypeDisplay {
    row.try_fold(RowTypeDisplay::Data, |_acc, column| {
        if column
            .as_string()
            .ok_or(RowTypeDisplay::Data)?
            .chars()
            .all(|c| c == '-' || c == ' ')
        {
            Ok(RowTypeDisplay::Separator)
        } else {
            Err(RowTypeDisplay::Data)
        }
    })
    .unwrap_or(RowTypeDisplay::Data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::columns_width::{Column, ColumnType};
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct IonFormatTestCase {
        raw: &'static str,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct DataOrSeparatorTestCase {
        row: Row,
        expected: RowTypeDisplay,
    }

    #[derive(Debug)]
    struct RowDisplayTestCase {
        row: Row,
        row_type: RowTypeDisplay,
        columns_width: ColumnsWidth,
        expected: &'static str,
    }

    fn string(value: &str) -> Value {
        Value::String(value.into())
    }

    static ION_FORMAT_CASE: LazyLock<IonFormatTestCase> = LazyLock::new(|| IonFormatTestCase {
        raw: indoc! {r#"
            [ALPHA]
            name = "foo"

            [BETA]
            | num | text |
            |-----|------|
            | 1   | A    |
            | 22  | B    |
        "#},
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
            raw: indoc! {r"
                [ALPHA]
                value = 7
            "},
            expected: indoc! {r"
                [ALPHA]
                value = 7

            "},
        });

    #[test_case(&*ION_FORMAT_CASE; "formats ion document")]
    #[test_case(&*ION_FORMAT_NON_STRING_DICTIONARY_CASE; "formats non string dictionary value")]
    fn format_ion_document(case: &IonFormatTestCase) {
        let ion = case.raw.parse::<Ion>().unwrap();
        assert_eq!(case.expected, format_ion(&ion));
        assert_eq!(case.expected, display(&ion).to_string());
    }

    static SEPARATOR_ROW_CASE: LazyLock<DataOrSeparatorTestCase> =
        LazyLock::new(|| DataOrSeparatorTestCase {
            row: vec![string("-----"), string(" ---- ")],
            expected: RowTypeDisplay::Separator,
        });
    static DATA_ROW_CASE: LazyLock<DataOrSeparatorTestCase> =
        LazyLock::new(|| DataOrSeparatorTestCase {
            row: vec![string("A"), Value::Integer(1)],
            expected: RowTypeDisplay::Data,
        });
    static NON_SEPARATOR_STRING_ROW_CASE: LazyLock<DataOrSeparatorTestCase> =
        LazyLock::new(|| DataOrSeparatorTestCase {
            row: vec![string("-----"), string("not-separator")],
            expected: RowTypeDisplay::Data,
        });
    static EMPTY_ROW_CASE: LazyLock<DataOrSeparatorTestCase> =
        LazyLock::new(|| DataOrSeparatorTestCase {
            row: vec![],
            expected: RowTypeDisplay::Data,
        });

    #[test_case(&*SEPARATOR_ROW_CASE; "separator row")]
    #[test_case(&*DATA_ROW_CASE; "data row")]
    #[test_case(&*NON_SEPARATOR_STRING_ROW_CASE; "non separator string row")]
    #[test_case(&*EMPTY_ROW_CASE; "empty row")]
    fn detects_row_type(case: &DataOrSeparatorTestCase) {
        assert_eq!(case.expected, data_or_separator(case.row.iter()));
    }

    static HEADER_ROW_DISPLAY_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
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
            row: vec![],
            row_type: RowTypeDisplay::Header,
            columns_width: ColumnsWidth::new(vec![]),
            expected: "",
        });
    static EMPTY_SEPARATOR_ROW_DISPLAY_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            row: vec![],
            row_type: RowTypeDisplay::Separator,
            columns_width: ColumnsWidth::new(vec![]),
            expected: "",
        });
    static EMPTY_DATA_ROW_DISPLAY_CASE: LazyLock<RowDisplayTestCase> =
        LazyLock::new(|| RowDisplayTestCase {
            row: vec![],
            row_type: RowTypeDisplay::Data,
            columns_width: ColumnsWidth::new(vec![]),
            expected: "",
        });

    #[test_case(&*HEADER_ROW_DISPLAY_CASE; "header row")]
    #[test_case(&*SEPARATOR_ROW_DISPLAY_CASE; "separator row")]
    #[test_case(&*DATA_ROW_DISPLAY_CASE; "data row")]
    #[test_case(&*HEADER_ROW_DISPLAY_EVEN_TEXT_CASE; "header row with even text")]
    #[test_case(
        &*DATA_ROW_DISPLAY_TEXT_THEN_NUMBER_CASE;
        "data row with text first and number next"
    )]
    #[test_case(&*EMPTY_HEADER_ROW_DISPLAY_CASE; "empty header row")]
    #[test_case(&*EMPTY_SEPARATOR_ROW_DISPLAY_CASE; "empty separator row")]
    #[test_case(&*EMPTY_DATA_ROW_DISPLAY_CASE; "empty data row")]
    fn display_row(case: &RowDisplayTestCase) {
        let row_display = RowDisplay {
            columns_width: &case.columns_width,
            row: &case.row,
            row_type: case.row_type,
        };
        assert_eq!(case.expected, row_display.to_string());
    }
}
