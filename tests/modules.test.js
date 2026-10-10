const assert = require("assert");

// Test Logger
const Logger = require("../scripts/logger");
assert.strictEqual(typeof Logger.info, "function");
assert.strictEqual(typeof Logger.debug, "function");
assert.strictEqual(typeof Logger.error, "function");
assert.strictEqual(typeof Logger.success, "function");

// Test Store
const StoreModule = require("../scripts/store");
const store = StoreModule.createStore(StoreModule.appReducer, StoreModule.initialAppState);
assert.strictEqual(store.getState().auth.state, "login");
store.dispatch({ type: "AUTH_SET_STATE", payload: "main" });
assert.strictEqual(store.getState().auth.state, "main");
store.dispatch({ type: "CSL_SET_STYLE", payload: "ieee" });
assert.strictEqual(store.getState().csl.selectedStyle, "ieee");

// Test CslConverter
const CslConverter = require("../scripts/csl-converter");
const mendeleyDoc = {
    id: "test-id-123",
    type: "journal",
    title: "Quantum Entanglement",
    year: 2024,
    authors: [{ first_name: "Albert", last_name: "Einstein" }]
};
const csl = CslConverter.convertMendeleyToCSL(mendeleyDoc);
assert.strictEqual(csl.id, "test-id-123");
assert.strictEqual(csl.type, "article-journal");
assert.strictEqual(csl.title, "Quantum Entanglement");
assert.deepStrictEqual(csl.issued, { "date-parts": [[2024]] });
assert.strictEqual(csl.author[0].family, "Einstein");


const journalCsl = CslConverter.convertMendeleyToCSL({
    id: "journal-id",
    type: "journal",
    source: "Psychometrika",
    series: "Series 1"
});
assert.strictEqual(journalCsl["container-title"], "Psychometrika");
assert.strictEqual(journalCsl["collection-title"], "Series 1");
// Test CitationSelection label formatting
const CitationSelection = require("../scripts/citation-selection");
const pillLabel = CitationSelection.getCitationPillLabel({
    year: 2024,
    authors: [{ family: "Einstein" }]
});
assert.strictEqual(pillLabel, "Einstein 2024");

const pillLabelTwo = CitationSelection.getCitationPillLabel({
    year: 2024,
    authors: [{ family: "Einstein" }, { family: "Rosen" }]
});
assert.strictEqual(pillLabelTwo, "Einstein & Rosen 2024");

// Test LibraryView shouldLoadMore logic
const LibraryView = require("../scripts/library-view");
const container = { scrollTop: 1300, clientHeight: 500, scrollHeight: 2000 };
assert.strictEqual(LibraryView.shouldLoadMore(container, "login"), false);

// Test DocBuilderHelper
const DocBuilderHelper = require("../scripts/docbuilder-helper");
assert.strictEqual(typeof DocBuilderHelper.buildBibliographyScript, "function");
assert.strictEqual(typeof DocBuilderHelper.parseHtmlToRuns, "function");
assert.strictEqual(typeof DocBuilderHelper.decodeEntities, "function");
console.log("# All module tests passed successfully!");
