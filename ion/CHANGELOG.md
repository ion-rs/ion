# Changelog

This changelog tracks releases of the `ion` crate only.
Repository and workspace maintenance changes are intentionally omitted.

## Unreleased

## 0.13.0

### Changed

- Store `Dictionary` and top-level `Sections` keys as `Box<str>` to reduce per-key overhead

### Breaking changes

- `Dictionary` is now `BTreeMap<Box<str>, Value>` (or `IndexMap<Box<str>, Value>` with `dictionary-indexmap`)
- `Sections` is now `BTreeMap<Box<str>, Section>` (or `IndexMap<Box<str>, Section>` with `dictionary-indexmap`)
- `Ion::get_key_value()` now returns `Option<(&str, &Section)>`
- `Ion::iter()` now yields `(&str, &Section)`

## 0.12.0

### Changed

- Extend the `dictionary-indexmap` backend to top-level `Sections`, so section iteration and serialization order now follow the selected backend too

### Breaking changes

- With `dictionary-indexmap` enabled, top-level section iteration and document serialization now preserve insertion order instead of remaining sorted

## 0.11.0

### Changed

- Reduce string value allocation overhead by storing `Value::String` as `Box<str>`

### Breaking changes

- `Value::as_string()` now returns `Option<&str>` instead of `Option<&String>`

## 0.10.0

### Added

- Add optional `dictionary-indexmap` feature to use `indexmap::IndexMap` for `Dictionary`
- Add `ParserErrorKind` with typed parser error variants (`CannotReadValue`, `UnclosedArray`, `UnclosedDictionary`)
- Add `ParserError::kind()` for machine-readable parser error handling
- Add `Ion::get_key_value`

### Changed

- Add support for optional leading `-` in dictionary numeric values
- Switch benchmarks from nightly `test::Bencher` to stable `criterion`
- Update crate to Rust edition 2024
- Keep parser error descriptions human-facing while storing structured parser error kind internally
- Dictionary display and serialization order now depend on the selected backend
  Default builds keep sorted `BTreeMap` behavior; `dictionary-indexmap` preserves insertion order

### Tests

- Expand unit and integration coverage across parser, display, `Ion`, `Section`, `Value`, `FromIon`, and `FromRow`
- Add backend-specific tests for `BTreeMap` vs `IndexMap` dictionary ordering

### Benchmarks

- Add backend-specific benchmarks for `BTreeMap` vs `IndexMap` dictionary ordering and serialization behavior

## 0.9.1

- Fix a couple of formatting edge cases

## 0.9.0

- Add license
- Clean up API and remove `Writer`
- Rename repository from `ion_rs` to `ion`

## 0.8.9

- Support escaping `\` when reading cells and strings

## 0.8.6

- Optimize parser internals

## 0.8.5

- Remove unused `slice_pattern` feature
- Remove deprecated `try!` macro
- Use inclusive range syntax

## 0.8.1

- Add filtering of sections when loading Ion

## 0.8

- Drop unused and unfinished features
- Add `RustcDeserialize` support
- Remove the non-working validator

## 0.7.3

- Fix `Display` of `Value::String` within arrays to be enclosed in `"`

## 0.7.1

- Fix slice pattern support for `rustc 1.12.0-nightly (2ad5ed07f 2016-07-08)`
