# Change Log

All notable changes to the `ion-fmt` VS Code extension are documented in this file.

## [0.1.0] - 2026-03-30

- Initial Marketplace-ready release.
- Added first-party formatter integration for `.ion` files using local `ion-fmt`.
- Added typed formatting settings:
  - `ionFmt.dictionaryField`
  - `ionFmt.sectionSpacing`
  - `ionFmt.documentSpacing`
- Added raw style passthrough setting:
  - `ionFmt.style`
- Added command:
  - `Ion: Format Document with ion-fmt`
- Added unit and integration test coverage for formatter option handling.
