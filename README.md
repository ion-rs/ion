# ion &emsp; [![crates-badge]][crates-link] [![docs-badge]][docs-link]

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rust-lang/docs.rs/master/LICENSE)
[![ion CI](https://github.com/ion-rs/ion/actions/workflows/ion.yml/badge.svg?branch=master)](https://github.com/ion-rs/ion/actions/workflows/ion.yml)
[![ion-fmt CI](https://github.com/ion-rs/ion/actions/workflows/ion-fmt.yml/badge.svg?branch=master)](https://github.com/ion-rs/ion/actions/workflows/ion-fmt.yml)
[![workspace CI](https://github.com/ion-rs/ion/actions/workflows/workspace.yml/badge.svg?branch=master)](https://github.com/ion-rs/ion/actions/workflows/workspace.yml)

[crates-badge]: https://img.shields.io/crates/v/ion.svg
[crates-link]: https://crates.io/crates/ion
[docs-badge]: https://img.shields.io/badge/docs.rs-latest-informational
[docs-link]: https://docs.rs/ion

`ion` is a Rust crate for parsing section-based `*.ion` documents into strongly typed data structures you can query, transform, and serialize back.

## Why Use `ion` in Rust?

- Parse mixed documents containing section dictionaries and table rows.
- Work with typed values (`String`, `i64`, `f64`, `bool`, arrays, dictionaries).
- Keep stable output formatting and control dictionary ordering backend.
- Filter parsing by section when you only need part of a large file.

## Installation

Default dictionary backend (`BTreeMap`):

```toml
[dependencies]
ion = "0.10.0"
```

Insertion-ordered dictionaries with `IndexMap`:

```toml
[dependencies]
ion = { version = "0.10.0", features = ["dictionary-indexmap"] }
```

## Related Crates

- `ion-fmt`: formatting library and CLI for Ion documents.
  See [`ion-fmt/README.md`](ion-fmt/README.md) for usage and CLI examples.

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

`Dictionary` uses:

- `BTreeMap` by default
- `IndexMap` with `dictionary-indexmap` feature flag

This affects ordering in:

- `Value::Dictionary`
- section field serialization
- `Ion::to_string()`

Section names are still stored separately and remain ordered independently from the dictionary backend.

## Benchmark Results

The parser uses `BTreeMap` for `Dictionary` by default. You can switch to `IndexMap` with:

```bash
cargo bench --bench parse --features dictionary-indexmap
```

The table below compares Criterion's middle estimate from:

- `cargo bench --bench parse`
- `cargo bench --bench parse --features dictionary-indexmap`

These measurements are directional, not universal. Actual results depend on the document shape, dictionary density, machine, and compiler version.

|                  Benchmark                   |  `btree`  | `indexmap` |       Delta       |
|----------------------------------------------|-----------|------------|-------------------|
| `section_on_start_of_ion`                    | 1.5342 ms | 1.5925 ms  | `indexmap` +3.8%  |
| `section_on_end_of_ion`                      | 1.5427 ms | 1.5957 ms  | `indexmap` +3.4%  |
| `section_on_start_of_ion_tuned_parser`       | 1.4685 ms | 1.5044 ms  | `indexmap` +2.4%  |
| `section_on_start_of_ion_parser_no_prealloc` | 1.6767 ms | 1.7540 ms  | `indexmap` +4.6%  |
| `section_on_end_of_ion_tuned_parser`         | 1.4645 ms | 1.5166 ms  | `indexmap` +3.6%  |
| `section_on_end_of_ion_parser_no_prealloc`   | 1.6815 ms | 1.7576 ms  | `indexmap` +4.5%  |
| `parse_filtered/section_on_start_of_ion`     | 7.6819 us | 7.8456 us  | `indexmap` +2.1%  |
| `parse_filtered/section_on_end_of_ion`       | 486.19 us | 433.22 us  | `indexmap` -10.9% |
| `dictionary/to_string_hotel`                 | 1.6572 ms | 1.6462 ms  | `indexmap` -0.7%  |
| `dictionary/read_hotel`                      | 1.5732 ms | 1.6453 ms  | `indexmap` +4.6%  |

In these sequential runs, `BTreeMap` was still faster in most parser paths, while `IndexMap` was faster when filtered parsing found the accepted section near the end of the input and slightly faster for `to_string()` on the hotel sample.

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

## License

Licensed under the MIT license.

## Development

- This repository is a Cargo workspace (`ion`, `ion-fmt`).
- Shared dependency versions are defined at the workspace root `Cargo.toml`.
