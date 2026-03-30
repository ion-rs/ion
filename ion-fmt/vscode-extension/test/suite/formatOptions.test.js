"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const os = require("node:os");
const path = require("node:path");
const vscode = require("vscode");

// Input document intentionally includes:
// - multiline dictionary text (`query`)
// - table rows (to exercise section/document spacing behavior)
const RAW_INPUT = `[ALPHA]
query = "
    SELECT 1
"
| c |
|---|
| 1 |
`;

// Matrix of all supported typed `FormatOptions` values:
// 2 dictionary styles x 2 section spacings x 2 document spacings = 8 cases.
const FORMAT_OPTION_CASES = [
    {
        dictionaryField: "singleline",
        sectionSpacing: "newline",
        documentSpacing: "end-newline",
    },
    {
        dictionaryField: "singleline",
        sectionSpacing: "newline",
        documentSpacing: "additional-end-newline",
    },
    {
        dictionaryField: "singleline",
        sectionSpacing: "additional-newline",
        documentSpacing: "end-newline",
    },
    {
        dictionaryField: "singleline",
        sectionSpacing: "additional-newline",
        documentSpacing: "additional-end-newline",
    },
    {
        dictionaryField: "multiline",
        sectionSpacing: "newline",
        documentSpacing: "end-newline",
    },
    {
        dictionaryField: "multiline",
        sectionSpacing: "newline",
        documentSpacing: "additional-end-newline",
    },
    {
        dictionaryField: "multiline",
        sectionSpacing: "additional-newline",
        documentSpacing: "end-newline",
    },
    {
        dictionaryField: "multiline",
        sectionSpacing: "additional-newline",
        documentSpacing: "additional-end-newline",
    },
];

function caseName(testCase) {
    return `dictionaryField=${testCase.dictionaryField}, sectionSpacing=${testCase.sectionSpacing}, documentSpacing=${testCase.documentSpacing}`;
}

function expectedOutput(testCase) {
    // Reproduce the formatter output expected for each option combination.
    const queryField =
        testCase.dictionaryField === "singleline"
            ? `query = "\\n    SELECT 1\\n"`
            : `query = "\n    SELECT 1\n"`;
    const sectionBreak = testCase.sectionSpacing === "additional-newline" ? "\n\n" : "\n";
    const trailingBreak =
        testCase.documentSpacing === "additional-end-newline" ? "\n\n" : "\n";
    return `[ALPHA]\n${queryField}${sectionBreak}| c |\n|---|\n| 1 |\n${trailingBreak}`;
}

async function updateFormatterSettings(uri, settings) {
    const configuration = vscode.workspace.getConfiguration("ionFmt", uri);
    // Force deterministic formatter behavior for each test case.
    await configuration.update("executablePath", settings.executablePath, vscode.ConfigurationTarget.Global);
    await configuration.update("timeoutMs", 30_000, vscode.ConfigurationTarget.Global);
    await configuration.update("style", [], vscode.ConfigurationTarget.Global);
    await configuration.update("dictionaryField", settings.dictionaryField, vscode.ConfigurationTarget.Global);
    await configuration.update("sectionSpacing", settings.sectionSpacing, vscode.ConfigurationTarget.Global);
    await configuration.update("documentSpacing", settings.documentSpacing, vscode.ConfigurationTarget.Global);
}

suite("ion-fmt extension integration: format options", () => {
    let tempDir = "";
    let testFilePath = "";
    const ionFmtBinaryPath = process.env.ION_FMT_BIN || "";

    suiteSetup(async () => {
        assert.ok(ionFmtBinaryPath.length > 0, "ION_FMT_BIN environment variable is not set.");
        // Use a temporary document to avoid mutating repository files.
        tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "ion-fmt-vscode-tests-"));
        testFilePath = path.join(tempDir, "format-options.integration.ion");

        const extension = vscode.extensions.getExtension("ion-rs.ion-fmt-vscode");
        assert.ok(extension, "Unable to locate extension 'ion-rs.ion-fmt-vscode'.");
        await extension.activate();
    });

    suiteTeardown(async () => {
        try {
            await fs.rm(tempDir, { recursive: true, force: true });
        } catch {
            // Temporary directory cleanup can fail on locked files.
        }
    });

    for (const testCase of FORMAT_OPTION_CASES) {
        test(caseName(testCase), async () => {
            await fs.writeFile(testFilePath, RAW_INPUT, "utf8");

            const uri = vscode.Uri.file(testFilePath);
            await updateFormatterSettings(uri, {
                executablePath: ionFmtBinaryPath,
                ...testCase,
            });

            let document = await vscode.workspace.openTextDocument(uri);
            await vscode.window.showTextDocument(document);

            // Execute formatting through VS Code's provider path, not by calling the CLI directly.
            const edits = await vscode.commands.executeCommand(
                "vscode.executeFormatDocumentProvider",
                uri,
                { tabSize: 4, insertSpaces: true },
            );

            assert.ok(Array.isArray(edits), "Formatting provider did not return an edit list.");

            const workspaceEdit = new vscode.WorkspaceEdit();
            workspaceEdit.set(uri, edits);
            const applied = await vscode.workspace.applyEdit(workspaceEdit);
            assert.equal(applied, true, "Failed to apply formatting edits.");

            // Re-open and assert exact formatted output for this option combination.
            document = await vscode.workspace.openTextDocument(uri);
            assert.equal(document.getText(), expectedOutput(testCase));
        });
    }
});
