const assert = require("assert");
const DocBuilderHelper = require("../scripts/docbuilder-helper.js");

async function runTests() {
    console.log("# Testing DocBuilderHelper");

    // Test 1: Parse plain text entry
    const entries1 = DocBuilderHelper.parseHtmlToRuns("<div class=\"csl-entry\">Smith, J. (2020). Simple Book. Publisher.</div>");
    assert.strictEqual(entries1.length, 1, "Should have 1 paragraph");
    assert.strictEqual(entries1[0].runs.length, 1, "Should have 1 run");
    assert.strictEqual(entries1[0].runs[0].text, "Smith, J. (2020). Simple Book. Publisher.");
    assert.strictEqual(entries1[0].runs[0].italic, false);
    assert.strictEqual(entries1[0].runs[0].bold, false);

    // Test 2: Parse entry with italics and HTML entities
    const entries2 = DocBuilderHelper.parseHtmlToRuns(
        "<div class=\"csl-entry\">Furlanetto, T. S., &amp; Loss, J. F. (2016). Study. <i>World Journal of Orthopedics</i>, <i>7</i>(3), 190&ndash;206.</div>"
    );
    assert.strictEqual(entries2.length, 1);
    const runs2 = entries2[0].runs;
    assert.ok(runs2.length >= 3, "Should split text into normal and italic runs");
    
    // Check &amp; entity decoding
    assert.ok(runs2[0].text.includes("&"), "XML entity &amp; should decode to &");
    assert.strictEqual(runs2[0].italic, false);

    // Check italic run
    const italicRun = runs2.find(r => r.text === "World Journal of Orthopedics");
    assert.ok(italicRun, "Should find italicized journal run");
    assert.strictEqual(italicRun.italic, true);

    // Check endash entity decoding
    const volRun = runs2[runs2.length - 1];
    assert.ok(volRun.text.includes("–") || volRun.text.includes("-"), "Entity &ndash; decoded");

    // Test 3: Generate DocumentBuilder script with hanging indent
    const scriptWithIndent = DocBuilderHelper.buildBibliographyScript(
        ["<div class=\"csl-entry\">Author. (2020). <i>Title</i>.</div>"],
        { hangingIndent: true }
    );
    assert.ok(scriptWithIndent.includes("Api.CreateParagraph()"), "Script must create paragraph");
    assert.ok(scriptWithIndent.includes("SetIndLeft(720)"), "Must set left indent for hanging indent");
    assert.ok(scriptWithIndent.includes("SetIndFirstLine(-720)"), "Must set negative first line indent");
    assert.ok(scriptWithIndent.includes("SetItalic(true)"), "Must set italic on title run");
    assert.ok(scriptWithIndent.includes("r0_0.SetItalic(false)"), "Plain text run must explicitly disable inherited italics");
    assert.ok(scriptWithIndent.includes("r0_0.SetBold(false)"), "Plain text run must explicitly disable inherited bold");
    assert.ok(scriptWithIndent.includes("InsertContent("), "Must insert paragraphs into document");

    // Test 4: Generate DocumentBuilder script without hanging indent
    const scriptNoIndent = DocBuilderHelper.buildBibliographyScript(
        ["<div class=\"csl-entry\">Author. (2020). Title.</div>"],
        { hangingIndent: false }
    );
    assert.ok(!scriptNoIndent.includes("SetIndLeft"), "Should not set left indent when hangingIndent is false");

    // Test 5: Fallback script for plain text
    const fallbackScript = DocBuilderHelper.buildPlainTextScript("Plain bibliography text");
    assert.ok(fallbackScript.includes("AddText(\"Plain bibliography text\")"));
    assert.ok(fallbackScript.includes("InsertContent("));

    console.log("# All DocBuilderHelper tests passed successfully!");
}

runTests().catch(err => {
    console.error("Test failed:", err);
    process.exit(1);
});
