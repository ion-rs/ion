"use strict";

const assert = require("node:assert/strict");
const Module = require("node:module");

function loadInternals() {
    const originalLoad = Module._load;
    Module._load = function patchedLoad(request, parent, isMain) {
        if (request === "vscode") {
            return {};
        }

        return originalLoad.call(this, request, parent, isMain);
    };

    try {
        return require("../../extension").__internal;
    } finally {
        Module._load = originalLoad;
    }
}

function createConfiguration(values) {
    return {
        get(key, defaultValue) {
            return Object.prototype.hasOwnProperty.call(values, key) ? values[key] : defaultValue;
        },
    };
}

const { getStyleArgs } = loadInternals();

suite("getStyleArgs", () => {
    test("returns typed style options followed by raw style entries", () => {
        const configuration = createConfiguration({
            dictionaryField: "multiline",
            sectionSpacing: "newline",
            documentSpacing: "end-newline",
            style: ["dictionary-field=singleline", "document-spacing=additional-end-newline"],
        });

        assert.deepEqual(getStyleArgs(configuration), [
            "--style",
            "dictionary-field=multiline",
            "--style",
            "section-spacing=newline",
            "--style",
            "document-spacing=end-newline",
            "--style",
            "dictionary-field=singleline",
            "--style",
            "document-spacing=additional-end-newline",
        ]);
    });

    test("ignores blank or non-string typed options", () => {
        const configuration = createConfiguration({
            dictionaryField: "   ",
            sectionSpacing: null,
            documentSpacing: 1,
            style: [],
        });

        assert.deepEqual(getStyleArgs(configuration), []);
    });

    test("trims typed and raw options", () => {
        const configuration = createConfiguration({
            dictionaryField: " multiline ",
            sectionSpacing: " newline ",
            documentSpacing: " end-newline ",
            style: [" custom-option=yes ", "   "],
        });

        assert.deepEqual(getStyleArgs(configuration), [
            "--style",
            "dictionary-field=multiline",
            "--style",
            "section-spacing=newline",
            "--style",
            "document-spacing=end-newline",
            "--style",
            "custom-option=yes",
        ]);
    });

    test("ignores invalid raw style container and entries", () => {
        const nonArrayStyleConfiguration = createConfiguration({
            dictionaryField: "singleline",
            sectionSpacing: "additional-newline",
            documentSpacing: "additional-end-newline",
            style: "not-an-array",
        });

        assert.deepEqual(getStyleArgs(nonArrayStyleConfiguration), [
            "--style",
            "dictionary-field=singleline",
            "--style",
            "section-spacing=additional-newline",
            "--style",
            "document-spacing=additional-end-newline",
        ]);

        const mixedStyleConfiguration = createConfiguration({
            dictionaryField: "singleline",
            sectionSpacing: "additional-newline",
            documentSpacing: "additional-end-newline",
            style: [1, "", "    ", "section-spacing=newline"],
        });

        assert.deepEqual(getStyleArgs(mixedStyleConfiguration), [
            "--style",
            "dictionary-field=singleline",
            "--style",
            "section-spacing=additional-newline",
            "--style",
            "document-spacing=additional-end-newline",
            "--style",
            "section-spacing=newline",
        ]);
    });
});
