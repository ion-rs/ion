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
//! - [`format_str_with_options`] formats a raw Ion string with explicit style options
//! - [`check_str_with_options`] checks whether a raw Ion string is already formatted
//! - [`format_file_with_options`] formats a file without rewriting it
//! - [`write_formatted_file_with_options`] rewrites a file in place when formatting changes it
//! - [`display_with_options`] and [`format_ion_with_options`] operate on a pre-parsed [`ion::Ion`] value
//! - [`DictionaryDisplay`] and [`DictionaryFieldDisplay`] render dictionary-only output
//! - [`FormatOptions`] controls style decisions such as dictionary string rendering,
//!   section spacing, and document trailing newlines
//!
//! # Examples
//!
//! Formatting a string:
//!
//! ```rust
//! use ion_fmt::{FormatOptions, format_str_with_options};
//!
//! let raw = "[A]\n[B]\n";
//! let formatted = format_str_with_options(raw, FormatOptions::default()).unwrap();
//! assert_eq!("[A]\n\n[B]\n\n", formatted);
//! ```
//!
//! Checking formatting:
//!
//! ```rust
//! use ion_fmt::{FormatOptions, check_str_with_options};
//!
//! assert!(!check_str_with_options("[A]\n[B]\n", FormatOptions::default()).unwrap());
//! assert!(check_str_with_options("[A]\n\n[B]\n\n", FormatOptions::default()).unwrap());
//! ```
//!
//! Formatting a parsed document:
//!
//! ```rust
//! use ion::Ion;
//! use ion_fmt::{
//!     DictionaryOptions, DocumentOptions, DocumentSpacing, FieldStyle, FormatOptions,
//!     SectionOptions, SectionSpacing, format_ion_with_options,
//! };
//!
//! let ion: Ion = "[A]\n[B]\n".parse().unwrap();
//! let formatted = format_ion_with_options(
//!     &ion,
//!     FormatOptions {
//!         dictionary: DictionaryOptions {
//!             field: FieldStyle::Singleline,
//!         },
//!         section: SectionOptions {
//!             spacing: SectionSpacing::NewLine,
//!         },
//!         document: DocumentOptions {
//!             spacing: DocumentSpacing::EndNewLine,
//!         },
//!     },
//! );
//! assert_eq!("[A]\n\n[B]\n\n", formatted);
//! ```
#![warn(missing_docs)]

mod columns_width;
mod display;
mod error;

/// Display wrapper and helpers used to render formatted Ion output.
pub use display::{
    DictionaryDisplay, DictionaryFieldDisplay, DictionaryOptions, DocumentOptions, DocumentSpacing,
    FieldStyle, FormatOptions, IonDisplay, SectionOptions, SectionSpacing, display_with_options,
    format_ion_with_options,
};
/// Error type returned by file-oriented formatting APIs.
pub use error::FormatError;

use ion::Ion;
use std::fs;
use std::path::Path;

/// Result returned by [`format_file_with_options`].
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

/// Formats an Ion string using explicit formatting options.
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
pub fn format_str_with_options(raw: &str, options: FormatOptions) -> Result<String, ion::IonError> {
    raw.parse::<Ion>()
        .map(|ion| format_ion_with_options(&ion, options))
}

/// Checks whether an Ion string is already formatted for explicit options.
///
/// Returns `Ok(true)` when [`format_str_with_options`] would return exactly
/// the same bytes.
///
/// # Errors
///
/// Returns parser errors when `raw` is not a valid Ion document.
pub fn check_str_with_options(raw: &str, options: FormatOptions) -> Result<bool, ion::IonError> {
    Ok(format_str_with_options(raw, options)? == raw)
}

/// Reads a file, formats its content with explicit options, and reports if any
/// change is needed.
///
/// This function does not write to disk.
///
/// # Errors
///
/// Returns I/O errors when the file cannot be read and parser errors when its
/// content is invalid Ion.
pub fn format_file_with_options(
    path: impl AsRef<Path>,
    options: FormatOptions,
) -> Result<FormatResult, FormatError> {
    let raw = fs::read_to_string(path.as_ref())?;
    let formatted = format_str_with_options(&raw, options)?;
    let changed = raw != formatted;

    Ok(FormatResult { formatted, changed })
}

/// Formats a file in place using explicit formatting options.
///
/// Returns `Ok(true)` when the file was rewritten with formatted content.
///
/// # Errors
///
/// Returns I/O errors when the file cannot be read/written and parser errors
/// when its content is invalid Ion.
pub fn write_formatted_file_with_options(
    path: impl AsRef<Path>,
    options: FormatOptions,
) -> Result<bool, FormatError> {
    let path = path.as_ref();
    let result = format_file_with_options(path, options)?;

    if result.changed {
        fs::write(path, result.formatted)?;
    }

    Ok(result.changed)
}

#[cfg(test)]
mod tests {
    use super::{
        DictionaryOptions, DocumentOptions, DocumentSpacing, FieldStyle, FormatOptions,
        FormatResult, SectionOptions, SectionSpacing, check_str_with_options,
        format_file_with_options, format_str_with_options, write_formatted_file_with_options,
    };
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct FormatStringTestCase {
        description: &'static str,
        raw: &'static str,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct FormatStringWithOptionsTestCase {
        description: &'static str,
        raw: &'static str,
        options: FormatOptions,
        expected: &'static str,
    }

    #[derive(Debug)]
    struct CheckStringTestCase {
        description: &'static str,
        raw: &'static str,
        expected_is_formatted: bool,
    }

    static FORMAT_STRING_CASE: LazyLock<FormatStringTestCase> =
        LazyLock::new(|| FormatStringTestCase {
            description: "formats string input",
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
    static FORMAT_MULTILINE_DICTIONARY_STRING_CASE: LazyLock<FormatStringTestCase> =
        LazyLock::new(|| FormatStringTestCase {
            description: "formats multiline dictionary string value",
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

                [Table]
                |   col1   |
                |--------|
                | name1 |
                | name2 |
            "#},
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

                [Table]
                |  col1 |
                |-------|
                | name1 |
                | name2 |

            "#},
        });

    #[test_case(&*FORMAT_STRING_CASE)]
    #[test_case(&*FORMAT_MULTILINE_DICTIONARY_STRING_CASE)]
    fn default_format(case: &FormatStringTestCase) {
        assert_eq!(
            case.expected,
            format_str_with_options(case.raw, FormatOptions::default()).unwrap(),
            "{}",
            case.description
        );
    }

    const FORMAT_MULTILINE_DICTIONARY_STRING_WITH_MULTILINE_STYLE_CASE:
        FormatStringWithOptionsTestCase = FormatStringWithOptionsTestCase {
        description: "formats multiline dictionary string value with multiline style option",
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

                [Table]
                |   col1   |
                |--------|
                | name1 |
                | name2 |
            "#},
        options: FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Multiline,
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
                select = "
                    SELECT column
                    FROM table t1
                    INNER JOIN t2
                        ON t1.id = t2.
                    WHERE t1.userid = {{ user_id }}
                    ORDER BY name ASC
                "

                [Table]
                |  col1 |
                |-------|
                | name1 |
                | name2 |

            "#},
    };

    const FORMAT_MULTILINE_DICTIONARY_STRING_WITH_SINGLELINE_STYLE_CASE:
        FormatStringWithOptionsTestCase = FormatStringWithOptionsTestCase {
        description: "formats multiline dictionary string value with singleline style option",
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

                [Table]
                |  col1 |
                |-------|
                | name1 |
                | name2 |

            "#},
        ..FORMAT_MULTILINE_DICTIONARY_STRING_WITH_MULTILINE_STYLE_CASE
    };
    const FORMAT_DICTIONARY_AND_TABLE_WITH_NEWLINE_SECTION_SPACING_CASE:
        FormatStringWithOptionsTestCase = FormatStringWithOptionsTestCase {
        description: "formats dictionary and table with single newline section spacing",
        raw: indoc! {r#"
                [ALPHA]
                name = "foo"
                | col |
                |-----|
                | x   |
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
                [ALPHA]
                name = "foo"
                | col |
                |-----|
                | x   |

            "#},
    };
    const FORMAT_DICTIONARY_AND_TABLE_WITH_ADDITIONAL_NEWLINE_SECTION_SPACING_CASE:
        FormatStringWithOptionsTestCase = FormatStringWithOptionsTestCase {
        description: "formats dictionary and table with extra spacing by default",
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

                | col |
                |-----|
                | x   |

            "#},
        ..FORMAT_DICTIONARY_AND_TABLE_WITH_NEWLINE_SECTION_SPACING_CASE
    };
    const FORMAT_TABLE_ONLY_WITH_NEWLINE_SECTION_SPACING_CASE: FormatStringWithOptionsTestCase =
        FormatStringWithOptionsTestCase {
            description: "does not add section spacing when dictionary is empty",
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
    const FORMAT_DOCUMENT_WITH_ADDITIONAL_NEWLINE_SPACING_CASE: FormatStringWithOptionsTestCase =
        FormatStringWithOptionsTestCase {
            description: "adds additional newline at end of document when configured",
            options: FormatOptions {
                dictionary: DictionaryOptions {
                    field: FieldStyle::Singleline,
                },
                section: SectionOptions {
                    spacing: SectionSpacing::NewLine,
                },
                document: DocumentOptions {
                    spacing: DocumentSpacing::AdditionalEndNewLine,
                },
            },
            expected: indoc! {r#"
                [ALPHA]
                name = "foo"
                | col |
                |-----|
                | x   |


            "#},
            ..FORMAT_DICTIONARY_AND_TABLE_WITH_NEWLINE_SECTION_SPACING_CASE
        };

    #[test_case(&FORMAT_MULTILINE_DICTIONARY_STRING_WITH_MULTILINE_STYLE_CASE)]
    #[test_case(&FORMAT_MULTILINE_DICTIONARY_STRING_WITH_SINGLELINE_STYLE_CASE)]
    #[test_case(&FORMAT_DICTIONARY_AND_TABLE_WITH_ADDITIONAL_NEWLINE_SECTION_SPACING_CASE)]
    #[test_case(&FORMAT_DICTIONARY_AND_TABLE_WITH_NEWLINE_SECTION_SPACING_CASE)]
    #[test_case(&FORMAT_TABLE_ONLY_WITH_NEWLINE_SECTION_SPACING_CASE)]
    #[test_case(&FORMAT_DOCUMENT_WITH_ADDITIONAL_NEWLINE_SPACING_CASE)]
    fn format_with_options_cases(case: &FormatStringWithOptionsTestCase) {
        assert_eq!(
            case.expected,
            format_str_with_options(case.raw, case.options).unwrap(),
            "{}",
            case.description
        );
    }

    static CHECK_FALSE_CASE: LazyLock<CheckStringTestCase> =
        LazyLock::new(|| CheckStringTestCase {
            description: "not formatted",
            raw: indoc! {r"
            [A]
            [B]
        "},
            expected_is_formatted: false,
        });
    static CHECK_TRUE_CASE: LazyLock<CheckStringTestCase> = LazyLock::new(|| CheckStringTestCase {
        description: "already formatted",
        raw: indoc! {r"
            [A]

            [B]

        "},
        expected_is_formatted: true,
    });
    static CHECK_MULTILINE_DICTIONARY_STRING_CASE: LazyLock<CheckStringTestCase> =
        LazyLock::new(|| CheckStringTestCase {
            description: "multiline dictionary string already formatted",
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

                [Table]
                |  col1 |
                |-------|
                | name1 |
                | name2 |

            "#},
            expected_is_formatted: true,
        });

    #[test_case(&*CHECK_FALSE_CASE)]
    #[test_case(&*CHECK_TRUE_CASE)]
    #[test_case(&*CHECK_MULTILINE_DICTIONARY_STRING_CASE)]
    fn default_check(case: &CheckStringTestCase) {
        assert_eq!(
            case.expected_is_formatted,
            check_str_with_options(case.raw, FormatOptions::default()).unwrap(),
            "{}",
            case.description
        );
    }

    #[test]
    fn check_string_with_options_multiline_style() {
        let raw = indoc! {r#"
            [Data]
            select = "
                SELECT column
                FROM table t1
                INNER JOIN t2
                    ON t1.id = t2.
                WHERE t1.userid = {{ user_id }}
                ORDER BY name ASC
            "

            [Table]
            |  col1 |
            |-------|
            | name1 |
            | name2 |

        "#};
        let options = FormatOptions {
            dictionary: DictionaryOptions {
                field: FieldStyle::Multiline,
            },
            section: SectionOptions {
                spacing: SectionSpacing::NewLine,
            },
            document: DocumentOptions {
                spacing: DocumentSpacing::EndNewLine,
            },
        };

        assert_eq!(true, check_str_with_options(raw, options).unwrap());
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

        let result = format_file_with_options(&temp_path, FormatOptions::default()).unwrap();
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

        assert_eq!(
            true,
            write_formatted_file_with_options(&temp_path, FormatOptions::default()).unwrap()
        );
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
