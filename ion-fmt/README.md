# ion-fmt &emsp; [![crates-badge]][crates-link]

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rust-lang/docs.rs/master/LICENSE)
[![ion-fmt CI](https://github.com/ion-rs/ion/actions/workflows/ion-fmt.yml/badge.svg?branch=master)](https://github.com/ion-rs/ion/actions/workflows/ion-fmt.yml)

[crates-badge]: https://img.shields.io/crates/v/ion.svg
[crates-link]: https://crates.io/crates/ion-fmt

`ion-fmt` formats Ion documents from Rust code and from the terminal.

## Feature Flags

- `default`: uses `BTreeMap` in `ion` and prints dictionary keys in sorted order.
- `dictionary-indexmap`: uses `IndexMap` in `ion` and preserves insertion order.

Default examples in this file are validated by `trycmd`.
For `dictionary-indexmap` snapshots, see `README.indexmap.md`.

## Install

- Default backend (`BTreeMap`, sorted dictionary keys): `cargo install ion-fmt`
- `dictionary-indexmap` backend (insertion-order dictionary keys): `cargo install ion-fmt --features dictionary-indexmap`
- From a local checkout (default backend): `cargo install --path ion-fmt`
- From a local checkout (`dictionary-indexmap`): `cargo install --path ion-fmt --features dictionary-indexmap`

`ion-fmt` backend is selected at build/install time; one installed binary uses one backend.

## Library

```rust
use ion_fmt::format_str;

let raw = r#"
    [A]
    [B]
"#;

let formatted = format_str(raw).unwrap();
```

## CLI

```console
$ ion-fmt --help
Formats Ion files.

Build mode: default dictionary backend (BTreeMap, dictionary keys are sorted).

Usage: ion-fmt [COMMAND]

Commands:
  format  Format files in place or stdin
  check   Check formatting without writing changes
  stdout  Print formatted output to stdout
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

```console
$ ion-fmt --version
ion-fmt 0.11.0

$ ion-fmt -V
ion-fmt 0.11.0

```

```console
$ ion-fmt stdout tests/readme/unformatted.ion
[WITHOUT_HEADER]
batch = 7
enabled = false
source = "legacy"
| 1 | alpha | PL |  11.2 |  ok  |
|---|-------|----|-------|------|
| 3 | gamma | UK | 13.75 | ok   |

[WITH_HEADER]
active = true
priority = 10
title = "rate-plan"
|   day    | market | room |  net   | seats |
|----------|--------|------|--------|-------|
| 20260101 | PL     | DBL  |  120.5 |     3 |
| 20260102 | DE     | SGL  |     99 |    12 |
| 20260103 | UK     | APP  | 145.25 |     2 |


```

```console
$ ion-fmt check tests/readme/unformatted.ion
? 1
needs formatting: tests/readme/unformatted.ion

$ ion-fmt check tests/readme/formatted.ion

```

```console
$ cat tests/readme/unformatted.ion | ion-fmt
[WITHOUT_HEADER]
batch = 7
enabled = false
source = "legacy"
| 1 | alpha | PL |  11.2 |  ok  |
|---|-------|----|-------|------|
| 3 | gamma | UK | 13.75 | ok   |

[WITH_HEADER]
active = true
priority = 10
title = "rate-plan"
|   day    | market | room |  net   | seats |
|----------|--------|------|--------|-------|
| 20260101 | PL     | DBL  |  120.5 |     3 |
| 20260102 | DE     | SGL  |     99 |    12 |
| 20260103 | UK     | APP  | 145.25 |     2 |
```

The CLI arguments are implemented with `clap` derive and subcommands.
When no subcommand is provided, `ion-fmt` defaults to the `stdout` command (stdin -> stdout).

## Related Crates

- [`ion`](https://crates.io/crates/ion): library for Ion documents.
