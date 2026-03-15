# ion-fmt

This file is used by `trycmd` when tests run with `--features dictionary-indexmap`.

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
ion-fmt 0.10.1

$ ion-fmt -V
ion-fmt 0.10.1

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
