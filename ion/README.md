# ion &emsp; [![crates-badge]][crates-link] [![docs-badge]][docs-link]

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rust-lang/docs.rs/master/LICENSE)
[![ion CI](https://github.com/ion-rs/ion/actions/workflows/ion.yml/badge.svg?branch=master)](https://github.com/ion-rs/ion/actions/workflows/ion.yml)

[crates-badge]: https://img.shields.io/crates/v/ion.svg
[crates-link]: https://crates.io/crates/ion
[docs-badge]: https://img.shields.io/badge/docs.rs-latest-informational
[docs-link]: https://docs.rs/ion

`ion` is a Rust crate for parsing section-based `*.ion` documents into strongly typed data structures.

## Why Use `ion`?

- Parse mixed documents containing section dictionaries and table rows.
- Work with typed values (`String`, `i64`, `f64`, `bool`, arrays, dictionaries).
- Keep stable output formatting and choose section and dictionary ordering backend.
- Filter parsing by section when you only need part of a large file.

## Installation

Default ordered backend (`BTreeMap` for sections and dictionaries):

```toml
[dependencies]
ion = "0.13.0"
```

Insertion-ordered backend (`IndexMap` for sections and dictionaries):

```toml
[dependencies]
ion = { version = "0.13.0", features = ["dictionary-indexmap"] }
```

## Rust Quick Start

```rust
use ion::Ion;

let raw = r#"
    [HOTEL]
    name = "HOTEL"
    markets = ["PL", "DE"]

    [ROOMS]
    | code | capacity |
    |------|----------|
    | DBL  | 2        |
"#;

let ion: Ion = raw.parse().unwrap();

let hotel = ion.get("HOTEL").unwrap();
assert_eq!(
    Some("HOTEL"),
    hotel.get("name").and_then(|value| value.as_str())
);

let rooms = ion.get("ROOMS").unwrap();
assert_eq!(1, rooms.rows_without_header().len());
```

Parse only selected sections:

```rust
use ion::Ion;

let raw = r#"
    [IGNORED]
    key = "value"

    [KEPT]
    key = "kept"
"#;

let ion = Ion::from_str_filtered(raw, vec!["KEPT"]).unwrap();
assert!(ion.get("IGNORED").is_none());
assert!(ion.get("KEPT").is_some());
```

## Backend Choice

`Dictionary` and `Sections` use:

- `BTreeMap` by default.
- `IndexMap` with `dictionary-indexmap`.

This affects ordering in:

- top-level section iteration and serialization
- `Value::Dictionary`
- section field serialization
- `Ion::to_string()`

## Example Usage

The following examples demonstrate the flexibility and structure of `*.ion` files:

### Basic Section

```ini
[CONTRACT]
id = "HOTEL001"
name = "Hotel001"
currency = "EUR"
active = true
markets = ["DE", "PL"]
```

### Table Format

``` ini
[DEF.MEAL]
| code | description |
|------|-------------|
| RO   | Room Only   |

[DEF.ROOM]
| code | description |      occ       |
|------|-------------|----------------|
| SGL  | Single      | P1:2 A1:1 C0:1 |
| DBL  | Double      | P2:3 A2:2 C0:1 |
```

### Basic section with possible field types

```ini
[CONTRACT]
country = "Poland"                  // String
markets = ["PL", "DE", "UK"]        // Array
75042 = {                           // Dictionary
    view = "SV"                     // String
    loc  = ["M", "B"]               // Array
    dist = { beach_km = 4.1 }       // Dictionary
}
```

### Complex document built from few sections

```ini
[CONTRACT]
country = "Poland"
markets = ["PL", "DE", "UK"]
75042 = {
    view = "SV"
    loc  = ["M", "B"]
    dist = { beach_km = 4.1 }
}

[RATE.PLAN]
|       dates       | code |  description   |    rooms    | rules |
|-------------------|------|----------------|-------------|-------|
| 20200512:20200514 | BAR  | Best available | SGL,DBL,APP |       |
| 20200601:20200614 | BBV  | Best Bar View  | DBL,APP     |       |

# A `key-value` and `table` section
[RATE.BASE]
enable = true
legend = {
    UN = "unit night"
    RO = "room only"
}
|       dates       | charge | room | occ | meal |  amt   |
|-------------------|--------|------|-----|------|--------|
| 20161122:20170131 | UN     | APP  | A2  | RO   | 250.00 |
| 20161122:20170131 | UN     | APP  | A4  | RO   | 450.00 |
| 20161122:20170131 | UN     | APP  | A6  | RO   | 650.00 |
```

## Related Crates

- [`ion-fmt`](https://crates.io/crates/ion-fmt): formatter library and CLI for Ion documents.

## License

Licensed under the MIT license.
