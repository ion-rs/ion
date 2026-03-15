//! Column width and alignment inference used by the formatter.

use crate::display::{RowTypeDisplay, data_or_separator};
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
pub struct ColumnsWidth(Vec<Column>);

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

    pub(crate) fn get(&self, index: usize) -> Option<&Column> {
        self.0.get(index)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut Column> {
        self.0.get_mut(index)
    }

    pub(crate) fn insert(&mut self, index: usize, column: Column) {
        self.0.insert(index, column);
    }

    #[cfg(test)]
    pub(crate) fn new(values: Vec<Column>) -> Self {
        Self(values)
    }
}

impl<'a, T> FromIterator<T> for ColumnsWidth
where
    T: Iterator<Item = &'a Value> + Clone,
{
    fn from_iter<I: IntoIterator<Item = T>>(records: I) -> Self {
        let records_iter = records.into_iter();
        let size_hint = records_iter.size_hint();

        let mut columns = Self(Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0)));

        records_iter.for_each(|record| {
            let row_type = data_or_separator(record.clone());
            if matches!(row_type, RowTypeDisplay::Data) {
                extend(&mut columns, record);
            }
        });

        columns
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
    use pretty_assertions::assert_eq;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct ColumnTypeFromTextTestCase {
        text: &'static str,
        expected: ColumnType,
    }

    #[derive(Debug)]
    struct ColumnTypeTransitTestCase {
        current: ColumnType,
        new: ColumnType,
        expected: ColumnType,
    }

    static COLUMN_TYPE_NUMBER_CASE: LazyLock<ColumnTypeFromTextTestCase> =
        LazyLock::new(|| ColumnTypeFromTextTestCase {
            text: "-1.0",
            expected: ColumnType::Number,
        });
    static COLUMN_TYPE_TEXT_CASE: LazyLock<ColumnTypeFromTextTestCase> =
        LazyLock::new(|| ColumnTypeFromTextTestCase {
            text: "1A",
            expected: ColumnType::Text,
        });
    static COLUMN_TYPE_INVALID_NUMBER_CASE: LazyLock<ColumnTypeFromTextTestCase> =
        LazyLock::new(|| ColumnTypeFromTextTestCase {
            text: "--1",
            expected: ColumnType::Text,
        });

    #[test_case(&*COLUMN_TYPE_NUMBER_CASE; "number")]
    #[test_case(&*COLUMN_TYPE_TEXT_CASE; "text")]
    #[test_case(&*COLUMN_TYPE_INVALID_NUMBER_CASE; "invalid number")]
    fn column_type_from_text(case: &ColumnTypeFromTextTestCase) {
        assert_eq!(case.expected, ColumnType::from(case.text));
    }

    static TRANSIT_DEFAULT_TO_NUMBER_CASE: LazyLock<ColumnTypeTransitTestCase> =
        LazyLock::new(|| ColumnTypeTransitTestCase {
            current: ColumnType::Default,
            new: ColumnType::Number,
            expected: ColumnType::Number,
        });
    static TRANSIT_NUMBER_TO_TEXT_CASE: LazyLock<ColumnTypeTransitTestCase> =
        LazyLock::new(|| ColumnTypeTransitTestCase {
            current: ColumnType::Number,
            new: ColumnType::Text,
            expected: ColumnType::Text,
        });
    static TRANSIT_TEXT_TO_NUMBER_CASE: LazyLock<ColumnTypeTransitTestCase> =
        LazyLock::new(|| ColumnTypeTransitTestCase {
            current: ColumnType::Text,
            new: ColumnType::Number,
            expected: ColumnType::Text,
        });

    #[test_case(&*TRANSIT_DEFAULT_TO_NUMBER_CASE; "default to number")]
    #[test_case(&*TRANSIT_NUMBER_TO_TEXT_CASE; "number to text")]
    #[test_case(&*TRANSIT_TEXT_TO_NUMBER_CASE; "text stays text")]
    fn column_type_transit(case: &ColumnTypeTransitTestCase) {
        assert_eq!(case.expected, case.current.transit(case.new));
    }
}
