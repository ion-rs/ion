# ion-fmt

This file is used by `trycmd` when tests run with `--features dictionary-indexmap`.

## CLI

```console
$ ion-fmt --help
Formats Ion files.

Build mode: dictionary-indexmap (section names and dictionary keys preserve insertion order).

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
ion-fmt 0.11.1

$ ion-fmt -V
ion-fmt 0.11.1

```

```console
$ ion-fmt stdout tests/readme/unformatted.ion
[WITH_HEADER]
title = "rate-plan"
active = true
priority = 10
|   day    | market | room |  net   | seats |
|----------|--------|------|--------|-------|
| 20260101 | PL     | DBL  |  120.5 |     3 |
| 20260102 | DE     | SGL  |     99 |    12 |
| 20260103 | UK     | APP  | 145.25 |     2 |

[WITHOUT_HEADER]
source = "legacy"
enabled = false
batch = 7
| 1 | alpha | PL |  11.2 |  ok  |
|---|-------|----|-------|------|
| 3 | gamma | UK | 13.75 | ok   |


```

```console
$ ion-fmt check tests/readme/unformatted.ion
? 1
needs formatting: tests/readme/unformatted.ion

$ ion-fmt check tests/readme/formatted.ion

```

```console
$ cat tests/readme/unformatted.ion | ion-fmt
[WITH_HEADER]
title = "rate-plan"
active = true
priority = 10
|   day    | market | room |  net   | seats |
|----------|--------|------|--------|-------|
| 20260101 | PL     | DBL  |  120.5 |     3 |
| 20260102 | DE     | SGL  |     99 |    12 |
| 20260103 | UK     | APP  | 145.25 |     2 |

[WITHOUT_HEADER]
source = "legacy"
enabled = false
batch = 7
| 1 | alpha | PL |  11.2 |  ok  |
|---|-------|----|-------|------|
| 3 | gamma | UK | 13.75 | ok   |
```

Interactive shell example when no file paths and no piped stdin are provided:

```text
$ ion-fmt
No input provided for `stdout`. Pass one or more file paths or pipe Ion through stdin.
```

The CLI arguments are implemented with `clap` derive and subcommands.
When no subcommand is provided, `ion-fmt` defaults to the `stdout` command.
If stdin is interactive and no paths are passed, it exits with an error instead of waiting for input.
