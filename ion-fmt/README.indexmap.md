# ion-fmt

`ion-fmt` formats Ion documents from Rust code and from the terminal.

## Feature Flags

- Active mode in this file: `dictionary-indexmap` (insertion-order dictionaries).
- Default mode (`BTreeMap`, sorted keys) is documented in `README.md`.

This file is used by `trycmd` when tests run with `--features dictionary-indexmap`.

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

Build mode: dictionary-indexmap (dictionary keys preserve insertion order).

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
ion-fmt 0.10.0

$ ion-fmt -V
ion-fmt 0.10.0

```

```console
$ ion-fmt format --help
Format files in place or stdin

Usage: ion-fmt format [PATH]...

Arguments:
  [PATH]...  Ion file paths. Reads stdin when omitted

Options:
  -h, --help  Print help

```

```console
$ ion-fmt check --help
Check formatting without writing changes

Usage: ion-fmt check [PATH]...

Arguments:
  [PATH]...  Ion file paths. Reads stdin when omitted

Options:
  -h, --help  Print help

```

```console
$ ion-fmt stdout --help
Print formatted output to stdout

Usage: ion-fmt stdout [PATH]...

Arguments:
  [PATH]...  Ion file paths. Reads stdin when omitted

Options:
  -h, --help  Print help

```

```console
$ ion-fmt stdout tests/readme/unformatted.ion
[WITHOUT_HEADER]
source = "legacy"
enabled = false
batch = 7
| 1 | alpha | PL |  11.2 |  ok  |
|---|-------|----|-------|------|
| 3 | gamma | UK | 13.75 | ok   |

[WITH_HEADER]
title = "rate-plan"
active = true
priority = 10
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
source = "legacy"
enabled = false
batch = 7
| 1 | alpha | PL |  11.2 |  ok  |
|---|-------|----|-------|------|
| 3 | gamma | UK | 13.75 | ok   |

[WITH_HEADER]
title = "rate-plan"
active = true
priority = 10
|   day    | market | room |  net   | seats |
|----------|--------|------|--------|-------|
| 20260101 | PL     | DBL  |  120.5 |     3 |
| 20260102 | DE     | SGL  |     99 |    12 |
| 20260103 | UK     | APP  | 145.25 |     2 |
```

The CLI arguments are implemented with `clap` derive and subcommands.
When no subcommand is provided, `ion-fmt` defaults to the `stdout` command (stdin -> stdout).
