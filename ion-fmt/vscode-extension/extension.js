"use strict";

// VS Code formatter bridge for .ion files.
// The extension sends in-memory document text to `ion-fmt stdout` via stdin and
// replaces the full document when formatter output differs.

const cp = require("node:child_process");
const vscode = require("vscode");

/**
 * Appends one `--style key=value` pair when the setting has a non-empty string value.
 */
function pushStyleArg(args, key, value) {
    if (typeof value !== "string") {
        return;
    }

    const trimmed = value.trim();
    if (trimmed.length === 0) {
        return;
    }

    args.push("--style", `${key}=${trimmed}`);
}

/**
 * Converts style-related settings into CLI arguments:
 * ["--style", "<key=value>", ...]
 *
 * Order:
 * 1) Structured FormatOptions-style settings (`dictionaryField`, `sectionSpacing`, `documentSpacing`)
 * 2) Additional raw `ionFmt.style` entries (applied last, so they can override)
 */
function getStyleArgs(configuration) {
    const args = [];
    pushStyleArg(args, "dictionary-field", configuration.get("dictionaryField", null));
    pushStyleArg(args, "section-spacing", configuration.get("sectionSpacing", null));
    pushStyleArg(args, "document-spacing", configuration.get("documentSpacing", null));

    const style = configuration.get("style", []);
    if (!Array.isArray(style)) {
        return args;
    }

    for (const entry of style) {
        if (typeof entry !== "string") {
            continue;
        }

        const trimmed = entry.trim();
        if (trimmed.length === 0) {
            continue;
        }

        args.push("--style", trimmed);
    }

    return args;
}

/**
 * Runs ion-fmt for one document and resolves with formatter output.
 *
 * Behavior:
 * - Uses workspace-relative cwd when available.
 * - Applies configurable timeout (`ionFmt.timeoutMs`).
 * - Aborts process on VS Code cancellation.
 */
function runIonFmt(source, document, token) {
    const configuration = vscode.workspace.getConfiguration("ionFmt", document.uri);
    const executablePath = String(configuration.get("executablePath", "ion-fmt")).trim() || "ion-fmt";
    const args = [...getStyleArgs(configuration), "stdout"];
    const timeoutMs = Math.max(1, Number(configuration.get("timeoutMs", 10000)) || 10000);

    const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
    const cwd = workspaceFolder ? workspaceFolder.uri.fsPath : undefined;

    return new Promise((resolve, reject) => {
        const child = cp.spawn(executablePath, args, { cwd });
        let stdout = "";
        let stderr = "";
        let didFinish = false;
        const timeout = setTimeout(() => {
            if (didFinish) {
                return;
            }

            child.kill();
            reject(new Error(`ion-fmt timed out after ${timeoutMs}ms`));
        }, timeoutMs);

        child.stdout.on("data", (chunk) => {
            stdout += chunk.toString();
        });

        child.stderr.on("data", (chunk) => {
            stderr += chunk.toString();
        });

        child.on("error", (error) => {
            didFinish = true;
            clearTimeout(timeout);
            reject(new Error(`failed to start '${executablePath}': ${error.message}`));
        });

        child.on("close", (code, signal) => {
            didFinish = true;
            clearTimeout(timeout);
            if (signal) {
                reject(new Error(`ion-fmt terminated by signal ${signal}`));
                return;
            }

            if (code === 0) {
                resolve(stdout);
                return;
            }

            const message = stderr.trim().length > 0 ? stderr.trim() : `ion-fmt exited with code ${code}`;
            reject(new Error(message));
        });

        token.onCancellationRequested(() => {
            didFinish = true;
            clearTimeout(timeout);
            child.kill();
        });

        child.stdin.on("error", () => {
            // Ignore stdin errors if the process exits early.
        });
        child.stdin.end(source);
    });
}

/**
 * Returns a range spanning entire document text.
 * Formatting replaces the whole document to mirror `ion-fmt` output exactly.
 */
function fullDocumentRange(document, source) {
    return new vscode.Range(document.positionAt(0), document.positionAt(source.length));
}

/**
 * Extension entrypoint:
 * - registers `ion` document formatter
 * - registers explicit "Ion: Format Document with ion-fmt" command
 */
function activate(context) {
    const outputChannel = vscode.window.createOutputChannel("ion-fmt");
    context.subscriptions.push(outputChannel);

    const provider = vscode.languages.registerDocumentFormattingEditProvider("ion", {
        async provideDocumentFormattingEdits(document, _options, token) {
            // External process execution is disabled in untrusted workspaces.
            if (!vscode.workspace.isTrusted) {
                outputChannel.appendLine("[ion-fmt] formatting skipped: workspace is not trusted");
                return [];
            }

            const source = document.getText();
            let formatted;
            try {
                formatted = await runIonFmt(source, document, token);
            } catch (error) {
                // Keep editor usable: report and return no edits on failure.
                const message = error instanceof Error ? error.message : String(error);
                outputChannel.appendLine(`[ion-fmt] ${message}`);
                void vscode.window.showErrorMessage(`ion-fmt failed: ${message}`);
                return [];
            }

            if (formatted === source) {
                return [];
            }

            return [vscode.TextEdit.replace(fullDocumentRange(document, source), formatted)];
        },
    });
    context.subscriptions.push(provider);

    const formatCommand = vscode.commands.registerCommand("ionFmt.formatDocument", async () => {
        if (!vscode.workspace.isTrusted) {
            void vscode.window.showWarningMessage(
                "ion-fmt requires a trusted workspace to run external binaries.",
            );
            return;
        }

        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            return;
        }

        if (editor.document.languageId !== "ion") {
            void vscode.window.showInformationMessage("The active document is not an Ion file.");
            return;
        }

        await vscode.commands.executeCommand("editor.action.formatDocument");
    });
    context.subscriptions.push(formatCommand);
}

function deactivate() {}

module.exports = {
    activate,
    deactivate,
    // Expose pure helpers for fast unit tests without spinning VS Code.
    __internal: {
        getStyleArgs,
        pushStyleArg,
    },
};
