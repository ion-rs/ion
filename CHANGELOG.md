# Changelog

## Unreleased

### Changed

- Convert the repository to a Cargo workspace and move the `ion` crate into `ion/`
- Make CI commands workspace-aware (`cargo test/clippy/fmt` via workspace scope)

### Breaking changes

- Repository layout changed for contributors: the crate manifest moved from `./Cargo.toml` to `./ion/Cargo.toml`
- Public library API remains unchanged

## 0.10.0

### Added

- Add optional `dictionary-indexmap` feature to use `indexmap::IndexMap` for `Dictionary`
- Add `Ion::get_key_value`
- Add `ParserErrorKind` with typed parser error variants (`CannotReadValue`, `UnclosedArray`, `UnclosedDictionary`)
- Add `ParserError::kind()` for machine-readable parser error handling

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

### Maintenance

- Fix clippy warnings for Rust 1.94

## 0.9.1

- Fixed a couple of formatting edge-cases

## 0.9.0

- Added license
- Cleaned up API, removed `Writer`
- Renamed repository from `ion_rs` to `ion`

## 0.8.9

- Support escape `\` character when reading cells and strings

## 0.8.6

- Optimize parser a bit

## 0.8.5

- Remove unused `slice_pattern` feature
- Remove deprecated `try!` macro
- Use inclusive range syntax

## 0.8.1

- Add filtering of sections when loading ion

## 0.8

- Drop unused / unfinished features
- RustcDeserialize support
- Validator (which wasn't working anyway)

## 0.7.3

- Fix `Display` of `Value::String` withing arrays to be enclosed in `"`

## 0.7.1

- Fix slice pattern for `rustc 1.12.0-nightly (2ad5ed07f 2016-07-08)`
