# Changelog

This changelog tracks releases of the `ion-fmt` crate only.
Repository and workspace maintenance changes are intentionally omitted.

## Unreleased

## 0.11.0

### Changed

- Make `ion-fmt --help` report the active dictionary build mode (`BTreeMap` sorted keys vs `dictionary-indexmap` insertion order)

## 0.10.0

### Added

- Add the `ion-fmt` crate with table and document formatting utilities for `ion::Ion`
- Add optional `dictionary-indexmap` support through the underlying `ion` crate backend

### Changed

- Move document formatting logic into `ion-fmt`
- Remove editor-specific naming from the moved formatter code
- Add rustdoc coverage for the library, formatter internals, and CLI entrypoint
- Update crate to Rust edition 2024
- Dictionary display order now depends on the selected backend
  Default builds keep sorted `BTreeMap` behavior; `dictionary-indexmap` preserves insertion order

### Tests

- Add backend-specific tests for `BTreeMap` vs `IndexMap` dictionary ordering
