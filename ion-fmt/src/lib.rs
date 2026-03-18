//! Ion document formatter as a reusable Rust library.
//!
//! `ion-fmt` parses Ion input with [`ion`] and renders it back with stable
//! spacing and aligned table columns.
//!
//! # Feature flags
//!
//! - default: uses `BTreeMap` through [`ion`], so section names and dictionary
//!   keys are formatted in sorted order
//! - `dictionary-indexmap`: uses `IndexMap` through [`ion`], so section names
//!   and dictionary keys preserve insertion order
//!
//! # Main entry points
//!
//! - [`format_str`] formats a raw Ion string
//! - [`check_str`] checks whether a raw Ion string is already formatted
//! - [`format_file`] formats a file without rewriting it
//! - [`write_formatted_file`] rewrites a file in place when formatting changes it
//! - [`display`] and [`format_ion`] operate on a pre-parsed [`ion::Ion`] value
//!
//! # Examples
//!
//! Formatting a string:
//!
//! ```rust
//! use ion_fmt::format_str;
//!
//! let raw = "[A]\n[B]\n";
//! let formatted = format_str(raw).unwrap();
//! assert_eq!("[A]\n\n[B]\n\n", formatted);
//! ```
//!
//! Checking formatting:
//!
//! ```rust
//! use ion_fmt::check_str;
//!
//! assert!(!check_str("[A]\n[B]\n").unwrap());
//! assert!(check_str("[A]\n\n[B]\n\n").unwrap());
//! ```
//!
//! Formatting a parsed document:
//!
//! ```rust
//! use ion::Ion;
//! use ion_fmt::format_ion;
//!
//! let ion: Ion = "[A]\n[B]\n".parse().unwrap();
//! assert_eq!("[A]\n\n[B]\n\n", format_ion(&ion));
//! ```
#![warn(missing_docs)]

mod columns_width;
mod display;
mod error;

/// Display wrapper and helpers used to render formatted Ion output.
pub use display::{IonDisplay, display, format_ion};
/// Error type returned by file-oriented formatting APIs.
pub use error::FormatError;

use ion::Ion;
use std::fs;
use std::path::Path;

/// Result returned by [`format_file`].
///
/// This lets callers inspect the formatted bytes before deciding whether to
/// write them back to disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormatResult {
    /// Formatted file content.
    pub formatted: String,
    /// `true` if formatting changed the original content.
    pub changed: bool,
}

/// Formats an Ion string into canonical `ion-fmt` output.
///
/// This keeps section ordering from parsed [`Ion`] and aligns table columns.
///
/// Section and dictionary ordering follow the active `ion` backend:
///
/// - default: sorted order
/// - `dictionary-indexmap`: insertion order
///
/// # Errors
///
/// Returns parser errors when `raw` is not a valid Ion document.
pub fn format_str(raw: &str) -> Result<String, ion::IonError> {
    raw.parse::<Ion>().map(|ion| format_ion(&ion))
}

/// Checks whether an Ion string is already formatted.
///
/// Returns `Ok(true)` when [`format_str`] would return exactly the same bytes.
///
/// # Errors
///
/// Returns parser errors when `raw` is not a valid Ion document.
pub fn check_str(raw: &str) -> Result<bool, ion::IonError> {
    Ok(format_str(raw)? == raw)
}

/// Reads a file, formats its content, and reports if any change is needed.
///
/// This function does not write to disk.
///
/// # Errors
///
/// Returns I/O errors when the file cannot be read and parser errors when its content is invalid Ion.
pub fn format_file(path: impl AsRef<Path>) -> Result<FormatResult, FormatError> {
    let raw = fs::read_to_string(path.as_ref())?;
    let formatted = format_str(&raw)?;
    let changed = raw != formatted;

    Ok(FormatResult { formatted, changed })
}

/// Formats a file in place.
///
/// Returns `Ok(true)` when the file was rewritten with formatted content.
///
/// # Errors
///
/// Returns I/O errors when the file cannot be read/written and parser errors when its content is invalid Ion.
pub fn write_formatted_file(path: impl AsRef<Path>) -> Result<bool, FormatError> {
    let path = path.as_ref();
    let result = format_file(path)?;

    if result.changed {
        fs::write(path, result.formatted)?;
    }

    Ok(result.changed)
}

#[cfg(test)]
mod tests {
    use super::{FormatResult, check_str, format_file, format_str, write_formatted_file};
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct FormatStringTestCase {
        raw: &'static str,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct CheckStringTestCase {
        raw: &'static str,
        expected_is_formatted: bool,
    }

    static FORMAT_STRING_CASE: LazyLock<FormatStringTestCase> =
        LazyLock::new(|| FormatStringTestCase {
            raw: indoc! {r#"
                [ONE]
                key = "foo"
                [TWO]
                | n | v |
                |---|---|
                | 1 | A |
            "#},
            expected: indoc! {r#"
                [ONE]
                key = "foo"

                [TWO]
                | n | v |
                |---|---|
                | 1 | A |

            "#},
        });

    #[test_case(&*FORMAT_STRING_CASE; "formats string input")]
    fn format_string(case: &FormatStringTestCase) {
        assert_eq!(case.expected, format_str(case.raw).unwrap());
    }

    static CHECK_FALSE_CASE: LazyLock<CheckStringTestCase> =
        LazyLock::new(|| CheckStringTestCase {
            raw: indoc! {r"
            [A]
            [B]
        "},
            expected_is_formatted: false,
        });
    static CHECK_TRUE_CASE: LazyLock<CheckStringTestCase> = LazyLock::new(|| CheckStringTestCase {
        raw: indoc! {r"
            [A]

            [B]

        "},
        expected_is_formatted: true,
    });

    #[test_case(&*CHECK_FALSE_CASE; "not formatted")]
    #[test_case(&*CHECK_TRUE_CASE; "already formatted")]
    fn check_string(case: &CheckStringTestCase) {
        assert_eq!(case.expected_is_formatted, check_str(case.raw).unwrap());
    }

    #[test]
    fn format_and_write_file() {
        let temp_path = std::env::temp_dir().join(format!(
            "ion-fmt-test-{}.ion",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let raw = indoc! {r"
            [X]
            [Y]
        "};
        fs::write(&temp_path, raw).unwrap();

        let result = format_file(&temp_path).unwrap();
        assert_eq!(
            FormatResult {
                formatted: indoc! {r"
                    [X]

                    [Y]

                "}
                .to_owned(),
                changed: true,
            },
            result
        );

        assert_eq!(true, write_formatted_file(&temp_path).unwrap());
        assert_eq!(
            indoc! {r"
                [X]

                [Y]

            "},
            fs::read_to_string(&temp_path).unwrap()
        );

        fs::remove_file(temp_path).unwrap();
    }
}
