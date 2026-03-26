//! Parser and data model for `.ion` documents.
//!
//! An Ion document is represented as [`Ion`], which contains named [`Section`] values.
//! Each section can hold:
//!
//! - a [`Dictionary`] of scalar, array, or nested dictionary fields
//! - tabular [`Row`] data
//!
//! The crate provides two entry points:
//!
//! - [`Ion`] for the high-level parsed document model
//! - [`Parser`] for lower-level iteration and error inspection
//!
//! # Feature flags
//!
//! - default: section and dictionary storage use `BTreeMap`, so iteration and
//!   serialization are sorted by key
//! - `dictionary-indexmap`: section and dictionary storage use `IndexMap`, so
//!   iteration and serialization preserve insertion order
//!
//! # Examples
//!
//! ```rust
//! use ion::{Ion, Value};
//!
//! let mut ion: Ion = r#"
//!     [APP]
//!     name = "demo"
//!     retries = 3
//! "#.parse()?;
//!
//! let app = ion.get_mut("APP").unwrap();
//! app.dictionary
//!     .insert("enabled".into(), Value::Boolean(true));
//!
//! assert_eq!(Some("demo"), app.get("name").and_then(Value::as_str));
//! # Ok::<(), ion::IonError>(())
//! ```
//!
//! # Ordering backend
//!
//! The selected backend affects both [`Dictionary`] and [`Sections`], which means it
//! changes:
//!
//! - top-level section iteration
//! - document serialization via [`std::string::ToString::to_string`]
//! - dictionary field iteration
//! - nested dictionary serialization
#![warn(missing_docs)]

#[macro_use]
mod ion;
mod parser;

pub use self::ion::*;
pub use self::parser::*;

/// Dictionary storage used by [`Section::dictionary`][crate::Section::dictionary].
///
/// The concrete map type depends on the `dictionary-indexmap` feature:
///
/// - default: `BTreeMap<Box<str>, Value>`
/// - `dictionary-indexmap`: `IndexMap<Box<str>, Value>`
#[cfg(feature = "dictionary-indexmap")]
pub type Dictionary = indexmap::IndexMap<Box<str>, Value>;
/// Dictionary storage used by [`Section::dictionary`][crate::Section::dictionary].
///
/// In default builds this is `BTreeMap<Box<str>, Value>`.
#[cfg(not(feature = "dictionary-indexmap"))]
pub type Dictionary = std::collections::BTreeMap<Box<str>, Value>;

/// Top-level section storage used by [`Ion`].
///
/// The concrete map type depends on the `dictionary-indexmap` feature:
///
/// - default: `BTreeMap<Box<str>, Section>`
/// - `dictionary-indexmap`: `IndexMap<Box<str>, Section>`
#[cfg(feature = "dictionary-indexmap")]
pub type Sections = indexmap::IndexMap<Box<str>, Section>;
/// Top-level section storage used by [`Ion`].
///
/// In default builds this is `BTreeMap<Box<str>, Section>`.
#[cfg(not(feature = "dictionary-indexmap"))]
pub type Sections = std::collections::BTreeMap<Box<str>, Section>;

/// A single table row stored inside a [`Section`].
pub type Row = Vec<Value>;
