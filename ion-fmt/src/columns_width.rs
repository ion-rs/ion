//! Column width and alignment inference used by the formatter.

use crate::display::RowTypeDisplay;
use ion::Value;
use std::cmp;

/// Kind of content inferred for a table column.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum ColumnType {
    /// Numeric-looking values, rendered right-aligned.
    Number,
    /// Text values, rendered left-aligned.
    Text,
    /// Unknown yet; resolved as rows are analyzed.
    #[default]
    Default,
}

/// Measured metadata for a single table column.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Column {
    /// Maximum visible width of the column content.
    pub width: usize,
    /// Inferred content type used for alignment.
    pub typ: ColumnType,
}

/// Width and type information for each column in a section table.
#[derive(Clone, Debug, Default)]
pub struct ColumnsWidth {
    width: Vec<Column>,
    row_types: Vec<RowTypeDisplay>,
}

impl ColumnType {
    const NUMBER_TYPE_CHARS: [char; 3] = ['.', '-', '+'];

    /// Returns `true` when values in this column should be right-aligned.
    #[must_use]
    pub fn is_number(self) -> bool {
        matches!(self, Self::Number)
    }

    /// Updates the current inferred type with a new observation.
    ///
    /// Once a column is inferred as text, it remains text.
    #[must_use]
    pub fn transit(self, new_type: Self) -> Self {
        match (self, new_type) {
            (Self::Default, _) | (Self::Number, Self::Text) => new_type,
            _ => self,
        }
    }
}

impl<T> From<T> for ColumnType
where
    T: AsRef<str>,
{
    /// Infers column type from a single cell text value.
    ///
    /// Strings made of digits and number punctuation (`.`, `-`, `+`) are
    /// treated as numeric candidates.
    fn from(value: T) -> Self {
        let mut iter = value.as_ref().chars().peekable();

        while let Some((left, right_opt)) = iter.next().map(|left| (left, iter.peek().copied())) {
            if !left.is_ascii_digit() {
                let left_allowed = Self::NUMBER_TYPE_CHARS.contains(&left);
                let right_number = right_opt.is_none_or(|right| right.is_ascii_digit());

                if !left_allowed || !right_number {
                    return Self::Text;
                }
            }
        }

        Self::Number
    }
}

impl Column {
    /// Returns `true` when this column should be rendered right-aligned.
    #[must_use]
    pub fn is_number(self) -> bool {
        self.typ.is_number()
    }
}

impl ColumnsWidth {
    /// Returns width for a column index or `0` if the column does not exist.
    #[must_use]
    pub fn width(&self, index: usize) -> usize {
        self.get(index).copied().unwrap_or_default().width
    }

    /// Returns full column metadata for an index or the default value.
    #[must_use]
    pub fn column(&self, index: usize) -> Column {
        self.get(index).copied().unwrap_or_default()
    }

    #[must_use]
    pub(crate) fn row_type(&self, row_idx: usize) -> RowTypeDisplay {
        self.row_types
            .get(row_idx)
            .copied()
            .unwrap_or(RowTypeDisplay::Data)
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Column> {
        self.width.get(index)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut Column> {
        self.width.get_mut(index)
    }

    pub(crate) fn insert(&mut self, index: usize, column: Column) {
        self.width.insert(index, column);
    }

    #[cfg(test)]
    pub(crate) fn new(values: Vec<Column>) -> Self {
        Self {
            width: values,
            row_types: Vec::new(),
        }
    }
}

impl<'a, T> FromIterator<T> for ColumnsWidth
where
    T: Iterator<Item = &'a Value> + Clone,
{
    fn from_iter<I: IntoIterator<Item = T>>(records: I) -> Self {
        let records_iter = records.into_iter();
        let size_hint = records_iter.size_hint();

        let mut columns = Self {
            width: Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0)),
            row_types: Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0)),
        };

        records_iter.enumerate().for_each(|(idx, record)| {
            let row_type = data_or_separator(record.clone());
            columns.row_types.push(row_type);
            if matches!(row_type, RowTypeDisplay::Data) {
                extend(&mut columns, record);
            }

            if idx == 1 && matches!(row_type, RowTypeDisplay::Separator) {
                // The second row is a separator, so the first row is a header.
                columns.row_types[0] = RowTypeDisplay::Header;
            }
        });

        columns
    }
}

/// Detects whether a row represents table data or a separator row.
///
/// Separator rows are string-only rows containing spaces and `-` characters.
fn data_or_separator<'a>(mut row: impl Iterator<Item = &'a Value>) -> RowTypeDisplay {
    if row.all(|column| {
        let Some(column) = column.as_string() else {
            return false;
        };
        column.chars().all(|c| c == '-' || c == ' ')
    }) {
        RowTypeDisplay::Separator
    } else {
        RowTypeDisplay::Data
    }
}

fn extend<'a, T>(columns: &mut ColumnsWidth, record: T)
where
    T: Iterator<Item = &'a Value>,
{
    record
        .enumerate()
        .for_each(|(column_idx, column)| set_max_width(columns, column_idx, format!("{column}")));
}

fn set_max_width<T: AsRef<str>>(columns: &mut ColumnsWidth, column_idx: usize, text: T) {
    if let Some(column) = columns.get_mut(column_idx) {
        column.width = cmp::max(column.width, text.as_ref().len());
        column.typ = column.typ.transit(text.as_ref().into());
    } else {
        columns.insert(
            column_idx,
            Column {
                width: text.as_ref().len(),
                typ: ColumnType::Default,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ion::Value;
    use pretty_assertions::assert_eq;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct ColumnTypeFromTextTestCase {
        description: &'static str,
        text: &'static str,
        expected: ColumnType,
    }

    #[derive(Debug)]
    struct ColumnTypeTransitTestCase {
        description: &'static str,
        current: ColumnType,
        new: ColumnType,
        expected: ColumnType,
    }

    #[derive(Debug)]
    struct DetectRowTypeTestCase {
        description: &'static str,
        row: Vec<Value>,
        expected: RowTypeDisplay,
    }

    fn string(value: &str) -> Value {
        Value::String(value.into())
    }

    static COLUMN_TYPE_NUMBER_CASE: LazyLock<ColumnTypeFromTextTestCase> =
        LazyLock::new(|| ColumnTypeFromTextTestCase {
            description: "number",
            text: "-1.0",
            expected: ColumnType::Number,
        });
    static COLUMN_TYPE_TEXT_CASE: LazyLock<ColumnTypeFromTextTestCase> =
        LazyLock::new(|| ColumnTypeFromTextTestCase {
            description: "text",
            text: "1A",
            expected: ColumnType::Text,
        });
    static COLUMN_TYPE_INVALID_NUMBER_CASE: LazyLock<ColumnTypeFromTextTestCase> =
        LazyLock::new(|| ColumnTypeFromTextTestCase {
            description: "invalid number",
            text: "--1",
            expected: ColumnType::Text,
        });

    #[test_case(&*COLUMN_TYPE_NUMBER_CASE)]
    #[test_case(&*COLUMN_TYPE_TEXT_CASE)]
    #[test_case(&*COLUMN_TYPE_INVALID_NUMBER_CASE)]
    fn column_type_from_text(case: &ColumnTypeFromTextTestCase) {
        assert_eq!(
            case.expected,
            ColumnType::from(case.text),
            "{}",
            case.description
        );
    }

    static TRANSIT_DEFAULT_TO_NUMBER_CASE: LazyLock<ColumnTypeTransitTestCase> =
        LazyLock::new(|| ColumnTypeTransitTestCase {
            description: "default to number",
            current: ColumnType::Default,
            new: ColumnType::Number,
            expected: ColumnType::Number,
        });
    static TRANSIT_NUMBER_TO_TEXT_CASE: LazyLock<ColumnTypeTransitTestCase> =
        LazyLock::new(|| ColumnTypeTransitTestCase {
            description: "number to text",
            current: ColumnType::Number,
            new: ColumnType::Text,
            expected: ColumnType::Text,
        });
    static TRANSIT_TEXT_TO_NUMBER_CASE: LazyLock<ColumnTypeTransitTestCase> =
        LazyLock::new(|| ColumnTypeTransitTestCase {
            description: "text stays text",
            current: ColumnType::Text,
            new: ColumnType::Number,
            expected: ColumnType::Text,
        });

    #[test_case(&*TRANSIT_DEFAULT_TO_NUMBER_CASE)]
    #[test_case(&*TRANSIT_NUMBER_TO_TEXT_CASE)]
    #[test_case(&*TRANSIT_TEXT_TO_NUMBER_CASE)]
    fn column_type_transit(case: &ColumnTypeTransitTestCase) {
        assert_eq!(
            case.expected,
            case.current.transit(case.new),
            "{}",
            case.description
        );
    }

    static SEPARATOR_ROW_CASE: LazyLock<DetectRowTypeTestCase> =
        LazyLock::new(|| DetectRowTypeTestCase {
            description: "separator row",
            row: vec![string("-----"), string(" ---- ")],
            expected: RowTypeDisplay::Separator,
        });
    static DATA_ROW_CASE: LazyLock<DetectRowTypeTestCase> =
        LazyLock::new(|| DetectRowTypeTestCase {
            description: "data row",
            row: vec![string("A"), Value::Integer(1)],
            expected: RowTypeDisplay::Data,
        });
    static NON_SEPARATOR_STRING_ROW_CASE: LazyLock<DetectRowTypeTestCase> =
        LazyLock::new(|| DetectRowTypeTestCase {
            description: "non separator string row",
            row: vec![string("-----"), string("not-separator")],
            expected: RowTypeDisplay::Data,
        });
    static MIXED_SEPARATOR_AND_TEXT_CASE: LazyLock<DetectRowTypeTestCase> =
        LazyLock::new(|| DetectRowTypeTestCase {
            description: "mixed separator and text row",
            row: vec![string("-----"), string("abc"), string("---")],
            expected: RowTypeDisplay::Data,
        });
    static TEXT_THEN_SEPARATORS_CASE: LazyLock<DetectRowTypeTestCase> =
        LazyLock::new(|| DetectRowTypeTestCase {
            description: "text then separators row",
            row: vec![string("abc"), string("----"), string("----")],
            expected: RowTypeDisplay::Data,
        });
    static EMPTY_ROW_CASE: LazyLock<DetectRowTypeTestCase> =
        LazyLock::new(|| DetectRowTypeTestCase {
            description: "empty row",
            row: vec![],
            expected: RowTypeDisplay::Separator,
        });

    #[test_case(&*SEPARATOR_ROW_CASE)]
    #[test_case(&*DATA_ROW_CASE)]
    #[test_case(&*NON_SEPARATOR_STRING_ROW_CASE)]
    #[test_case(&*MIXED_SEPARATOR_AND_TEXT_CASE)]
    #[test_case(&*TEXT_THEN_SEPARATORS_CASE)]
    #[test_case(&*EMPTY_ROW_CASE)]
    fn detects_row_type(case: &DetectRowTypeTestCase) {
        assert_eq!(
            case.expected,
            data_or_separator(case.row.iter()),
            "{}",
            case.description
        );
    }
}
