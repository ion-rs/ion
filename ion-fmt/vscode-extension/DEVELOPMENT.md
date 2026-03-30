# ion-fmt VS Code extension development

This guide covers contributor workflows for local development, tests, and
publishing.

## Code structure

- `extension.js`:
  - `getStyleArgs`: builds repeated `--style` args from typed style settings and `ionFmt.style`.
  - `runIonFmt`: spawns `ion-fmt stdout`, writes document text to stdin, applies timeout/cancellation, and returns stdout.
  - document formatter provider: skips untrusted workspaces, formats current text, and replaces the whole document only when output changed.
  - `ionFmt.formatDocument` command: runs VS Code's standard `editor.action.formatDocument` for the active Ion editor.
- `package.json`:
  - declares language registration for `.ion`
  - declares formatter settings schema (`executablePath`, typed style options, `style`, `timeoutMs`)
  - declares untrusted-workspace capability.
- `test/`:
  - integration test launcher and suites.

## Local development and debugging commands

Run these commands from the repository root unless stated otherwise.

| Command | Description |
| --- | --- |
| `cargo install --path ion-fmt --force` | Installs/updates `ion-fmt` binary from this repo for local testing. |
| `cargo install --path ion-fmt --force --features dictionary-indexmap` | Installs `ion-fmt` with the `dictionary-indexmap` backend for local testing. |
| `cd ion-fmt/vscode-extension && npm run lint` | Validates extension JavaScript syntax. |
| `code --extensionDevelopmentPath $HOME/sources/github/ion-rs/ion/ion-fmt/vscode-extension $HOME/sources/github/ion-rs/ion` | Launches a VS Code Extension Development Host with this local extension loaded on demand (no permanent install). |
| `code --uninstall-extension ion-rs.ion-fmt-vscode` | Uninstalls the extension from your local VS Code profile. |

## Testing

Run these commands from the repository root unless stated otherwise.

| Command | Description |
| --- | --- |
| `cd ion-fmt/vscode-extension && npm test` | Runs unit tests and integration tests (`test:unit` then `test:integration`). |
| `cd ion-fmt/vscode-extension && npm run test:integration` | Runs the same integration suite explicitly. |
| `cd ion-fmt/vscode-extension && ION_FMT_BIN="$HOME/.cargo/bin/ion-fmt" npm test` | Runs tests against a specific `ion-fmt` binary path. |

How tests work:

1. `test/runTest.js` resolves or builds the `ion-fmt` binary, clears `ELECTRON_RUN_AS_NODE`, and starts VS Code integration tests via `@vscode/test-electron`.
2. `test/suite/index.js` runs Mocha test files and prints per-test PASS/FAIL with a final summary.
3. `test/suite/formatOptions.test.js` applies extension settings and formats a real `.ion` document through VS Code's formatting provider.
4. The suite verifies all supported `FormatOptions` combinations:
   `dictionaryField (singleline|multiline) x sectionSpacing (newline|additional-newline) x documentSpacing (end-newline|additional-end-newline)`.

How tests are organized:

- `test/runTest.js`: test launcher and environment setup.
- `test/suite/index.js`: Mocha runner bootstrap.
- `test/suite/formatOptions.test.js`: integration matrix tests for formatter options.
- `test/fixture-workspace/`: fixture workspace placeholder for test launches.

## Packaging and publishing commands

Run these commands from the repository root unless stated otherwise.

| Command | Description |
| --- | --- |
| `cd ion-fmt/vscode-extension && npm run package` | Builds a `.vsix` package via `vsce package`. |
| `cd ion-fmt/vscode-extension && npx --yes @vscode/vsce ls` | Shows files that will be included in the VSIX/publish payload. |
| `cd ion-fmt/vscode-extension && npx --yes @vscode/vsce package` | Dry-run step 1: build/validate the publish artifact locally. |
| `cd ion-fmt/vscode-extension && npx --yes @vscode/vsce package && npx --yes @vscode/vsce ls` | Full dry-run workflow: package then inspect payload before publish. |
| `cd ion-fmt/vscode-extension && npx --yes @vscode/vsce publish -p "$VSCE_PAT"` | Publishes the current extension to Visual Studio Marketplace. |
| `cd ion-fmt/vscode-extension && npx --yes @vscode/vsce publish -i ion-fmt-vscode-0.1.0.vsix -p "$VSCE_PAT"` | Publishes a specific prebuilt VSIX artifact. |
| `cd ion-fmt/vscode-extension && npx --yes @vscode/vsce publish --help` | Displays publish command syntax and available options. |

Recommended publish sequence (verified commands):

1. `cd ion-fmt/vscode-extension && npm run -s lint`
2. `cd ion-fmt/vscode-extension && npm test`
3. `cd ion-fmt/vscode-extension && npx --yes @vscode/vsce package`
4. `cd ion-fmt/vscode-extension && npx --yes @vscode/vsce ls`
5. `cd ion-fmt/vscode-extension && npx --yes @vscode/vsce publish -p "$VSCE_PAT"`

Notes:

- `vsce publish --dry-run` is not supported in this setup; use `vsce package` + `vsce ls` as the dry-run workflow.
- Marketplace publish requires a publisher account matching `publisher` in `package.json` and a valid `VSCE_PAT`.
