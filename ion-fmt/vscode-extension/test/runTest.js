"use strict";

// VS Code integration test launcher.
// It ensures we have a runnable `ion-fmt` binary and starts the extension test host.

const cp = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const { runTests } = require("@vscode/test-electron");

function ionFmtBinaryName() {
    return process.platform === "win32" ? "ion-fmt.exe" : "ion-fmt";
}

function resolveIonFmtBinary() {
    if (process.env.ION_FMT_BIN && fs.existsSync(process.env.ION_FMT_BIN)) {
        return process.env.ION_FMT_BIN;
    }

    // Resolve from this repository first, then build if it is missing.
    const repoRoot = path.resolve(__dirname, "../../..");
    const candidate = path.join(repoRoot, "target", "debug", ionFmtBinaryName());
    if (fs.existsSync(candidate)) {
        return candidate;
    }

    cp.execSync("cargo +stable build -p ion-fmt", {
        cwd: repoRoot,
        stdio: "inherit",
    });

    if (!fs.existsSync(candidate)) {
        throw new Error(`Unable to find ion-fmt binary at '${candidate}' after build.`);
    }

    return candidate;
}

async function main() {
    const extensionDevelopmentPath = path.resolve(__dirname, "..");
    const extensionTestsPath = path.resolve(__dirname, "suite", "index.js");
    const ionFmtBin = resolveIonFmtBinary();
    // Some parent environments (for example, extension hosts) set this.
    // It makes Electron behave like Node and breaks VS Code test launches.
    delete process.env.ELECTRON_RUN_AS_NODE;

    await runTests({
        extensionDevelopmentPath,
        extensionTestsPath,
        extensionTestsEnv: {
            ION_FMT_BIN: ionFmtBin,
            ELECTRON_RUN_AS_NODE: "",
        },
        launchArgs: ["--disable-extensions"],
    });
}

main().catch((error) => {
    // eslint-disable-next-line no-console
    console.error("Failed to run extension tests:", error);
    process.exit(1);
});
