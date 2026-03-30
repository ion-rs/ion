"use strict";

// Mocha bootstrap for extension integration tests.
// Keeps output explicit so `npm test` is human-readable in CI and locally.

const Mocha = require("mocha");
const path = require("node:path");

function run() {
    const mocha = new Mocha({
        ui: "tdd",
        color: true,
        timeout: 120_000,
    });

    mocha.addFile(path.resolve(__dirname, "formatOptions.test.js"));

    return new Promise((resolve, reject) => {
        const runner = mocha.run((failures) => {
            const passes = runner.stats?.passes ?? 0;
            const tests = runner.stats?.tests ?? 0;
            // eslint-disable-next-line no-console
            console.log(`[ion-fmt-vscode] ${passes}/${tests} integration tests passed`);
            if (failures > 0) {
                reject(new Error(`${failures} test(s) failed.`));
                return;
            }
            resolve();
        });

        runner.on("pass", (test) => {
            // eslint-disable-next-line no-console
            console.log(`PASS ${test.fullTitle()}`);
        });
        runner.on("fail", (test, error) => {
            // eslint-disable-next-line no-console
            console.error(`FAIL ${test.fullTitle()}: ${error.message}`);
        });
    });
}

module.exports = {
    run,
};
