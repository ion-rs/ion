# Changelog

This changelog tracks releases of the `ion-fmt` crate only.
Repository and workspace maintenance changes are intentionally omitted.

## Unreleased

## 0.14.0

### Added

- Add section spacing option `section-spacing=newline|additional-newline` to control blank-line insertion between section dictionary fields and table rows
- Add document spacing option `document-spacing=end-newline|additional-end-newline` to control trailing newline behavior at end of document

### Changed

- Use `document-spacing=end-newline|additional-end-newline` names in CLI style values

### Breaking changes

- Change default section formatting to insert an empty line between dictionary fields and table rows
  (`section-spacing=additional-newline` is now the default; use `section-spacing=newline` to keep prior output)

## 0.13.0

### Added

- Add `--style dictionary-field=singleline|multiline` CLI option
- Add formatter options API (`FormatOptions`) with dictionary field style control
- Add public `DictionaryDisplay` and `DictionaryFieldDisplay` adapters

### Changed

- Make multiline the default dictionary-field rendering behavior

### Breaking changes

- Remove wrapper APIs in favor of options-explicit variants:
  `format_str` -> `format_str_with_options`,
  `check_str` -> `check_str_with_options`,
  `format_file` -> `format_file_with_options`,
  `write_formatted_file` -> `write_formatted_file_with_options`,
  `display` -> `display_with_options`,
  `format_ion` -> `format_ion_with_options`

## 0.12.0

### Changed

- Switch `ion-fmt` to `ion 0.12.0`
- With `dictionary-indexmap` enabled, formatted document output now preserves top-level section insertion order through the underlying `ion` backend

### Breaking changes

- `dictionary-indexmap` builds no longer sort top-level sections before formatting; output section order now follows input insertion order

## 0.11.1

### Fixed

- Make `ion-fmt` fail fast with a clear error when run without file paths and without piped stdin

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
