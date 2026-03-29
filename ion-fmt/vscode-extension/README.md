# ion-fmt VS Code extension

This extension provides first-party formatting support for `.ion` files using
the `ion-fmt` CLI from this repository.

## Features

- Registers `.ion` files under language id `ion`.
- Formats Ion documents by invoking `ion-fmt stdout` with file text via stdin.
- Sets default Ion editor behavior to use this formatter and enable format-on-save.

## Install

1. Install/refresh the formatter binary:
   `cargo install --path ion-fmt --force`
2. Build the extension VSIX:
   `cd ion-fmt/vscode-extension && npm run package`
3. Install it in VS Code:
   `code --install-extension ion-fmt/vscode-extension/ion-fmt-vscode-*.vsix --force`

## Settings

- `ionFmt.executablePath` (default: `ion-fmt`): path to the formatter binary.
- `ionFmt.dictionaryField` (default: `multiline`): maps to `--style dictionary-field=<value>`.
  Allowed values: `singleline`, `multiline`.
- `ionFmt.sectionSpacing` (default: `newline`): maps to `--style section-spacing=<value>`.
  Allowed values: `newline`, `additional-newline`.
- `ionFmt.documentSpacing` (default: `end-newline`): maps to `--style document-spacing=<value>`.
  Allowed values: `end-newline`, `additional-end-newline`.
- `ionFmt.style` (default: `[]`): additional raw `--style key=value` entries
  (applied after typed style settings; last value wins like CLI).
- `ionFmt.timeoutMs` (default: `10000`): timeout for one formatter invocation.

Example (User Settings JSON, top-level):

```json
"ionFmt.dictionaryField": "multiline",
"ionFmt.sectionSpacing": "newline",
"ionFmt.documentSpacing": "end-newline"
```

Example (language override under `"[ion]"`):

```json
"[ion]": {
  "ionFmt.dictionaryField": "multiline",
  "ionFmt.sectionSpacing": "newline",
  "ionFmt.documentSpacing": "end-newline",
  "editor.formatOnSave": true
}
```

To disable format-on-save:

```json
"[ion]": {
  "editor.formatOnSave": false
}
```

## VS Code commands

| Command | Description |
| --- | --- |
| `Ion: Format Document with ion-fmt` | Runs VS Code's format action for the active `.ion` editor using this extension. |
| `Format Document` | Standard VS Code formatting command; uses this extension when it is set as default formatter for Ion. |

## Security

- The extension executes the local `ion-fmt` binary and therefore requires a
  trusted VS Code workspace.

## Development

Development, testing, and publishing workflows are documented in
[`DEVELOPMENT.md`](https://github.com/ion-rs/ion/blob/main/ion-fmt/vscode-extension/DEVELOPMENT.md).
