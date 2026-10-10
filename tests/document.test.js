const assert = require("assert");
const { DocumentModule, InMemoryAdapter, OnlyOfficeAdapter } = require("../scripts/document.js");

async function runTests() {
    console.log("# Testing DocumentModule and InMemoryAdapter");

    // Test 1: Insert inline citation
    const inMem = new InMemoryAdapter();
    const doc = new DocumentModule(inMem);

    const testItems = [
        {
            id: "doc1",
            itemData: { id: "doc1", title: "Study of Architecture", author: [{ family: "Turing" }] },
            locator: "14",
            label: "page"
        }
    ];

    const ctrlId = await doc.insertCitation(testItems, "<b>(Turing &#38; Lovelace, 2026, p. 14)</b>", false);
    assert.ok(ctrlId, "Control ID must be returned");
    assert.strictEqual(inMem.controls.length, 1, "Should have 1 control in memory");
    assert.strictEqual(inMem.controls[0].text, "(Turing & Lovelace, 2026, p. 14)", "HTML tags must be stripped and entities decoded from rendered text");
    assert.strictEqual(inMem.controls[0].lock, 3, "Inline citation control must allow editing and deletion");
    assert.strictEqual(inMem.controls[0].placeHolderText, "(Turing & Lovelace, 2026, p. 14)", "PlaceHolderText must match citation text to prevent default 'Your text here'");
    assert.strictEqual(inMem.footnotesCount, 0, "No footnotes should be created for inline style");

    // Inline citations must remain inside the paragraph at the cursor.
    const paragraph = { elements: ["Sentence before citation."] };
    const controls = [];
    let insertedType;
    const host = {
        scope: {},
        plugin: {
            executeMethod(method, args, callback) {
                assert.strictEqual(method, "AddContentControl");
                insertedType = args[0];
                const control = {
                    tag: args[1].Tag,
                    text: "",
                    GetTag() {
                        return this.tag;
                    },
                    AddText(text) {
                        this.text += text;
                    }
                };
                controls.push(control);
                paragraph.elements.push(control);
                callback();
            },
            callCommand(command, close, recalculate, callback) {
                command();
                callback();
            }
        }
    };
    global.window = { Asc: host };
    global.Asc = host;
    global.Api = {
        GetDocument() {
            return {
                GetAllContentControls() {
                    return controls;
                }
            };
        }
    };
    try {
        const inlineAdapter = new OnlyOfficeAdapter();
        await inlineAdapter.addContentControl(2, { Tag: "citation-tag", Lock: 3 }, "(Turing, 2026)", false);
        assert.strictEqual(insertedType, 2, "Citation must use inline content control type");
        assert.strictEqual(paragraph.elements.length, 2, "Citation must be added to current paragraph, not a new paragraph");
        assert.strictEqual(paragraph.elements[1], controls[0]);
        assert.strictEqual(controls[0].tag, "citation-tag");
        assert.strictEqual(controls[0].text, "(Turing, 2026)");
    } finally {
        delete global.window;
        delete global.Asc;
        delete global.Api;
    }
    // Test 2: Read citations back
    const citations = await doc.getCitations();
    assert.strictEqual(citations.length, 1, "Should retrieve 1 citation record");
    assert.strictEqual(citations[0].internalId, ctrlId);
    assert.strictEqual(citations[0].citationItems.length, 1);
    assert.strictEqual(citations[0].citationItems[0].id, "doc1");
    assert.strictEqual(citations[0].citationItems[0].locator, "14");
    assert.strictEqual(citations[0].citationItems[0].itemData.title, "Study of Architecture");
    assert.strictEqual(citations[0].isNoteStyle, false);

    // Test 3: Insert note style citation (triggers footnote)
    const noteItems = [
        { id: "doc2", itemData: { id: "doc2", title: "Note Work" } }
    ];
    const noteCtrlId = await doc.insertCitation(noteItems, "Turing, <i>Note Work</i>", true);
    assert.strictEqual(inMem.controls.length, 2);
    assert.strictEqual(inMem.controls[1].html, "Turing, <i>Note Work</i>", "Note citation must preserve CSL formatting source");
    assert.strictEqual(inMem.footnotesCount, 1, "Footnote must be created for note style");

    // Test 4: Update citation text
    await doc.updateCitationText(ctrlId, "<i>(Turing, 2026, pp. 14-16)</i>");
    assert.strictEqual(inMem.controls[0].text, "(Turing, 2026, pp. 14-16)", "Updated text must have HTML tags stripped");

    await doc.updateCitationText(ctrlId, "(Putra &#38; Veronica, 2022)");
    assert.strictEqual(inMem.controls[0].text, "(Putra & Veronica, 2022)", "Refreshed citation text must decode HTML entities");

    // Test 5: Insert & get bibliography
    let bib = await doc.getBibliography();
    assert.strictEqual(bib, null, "Bibliography should not exist yet");

    const bibId = await doc.insertBibliography("<p>Turing. (2026). Study of Architecture.</p>", { hangingIndent: true });
    assert.ok(bibId);
    assert.strictEqual(inMem.controls[2].lock, 3, "Bibliography control must allow editing and deletion");
    assert.strictEqual(inMem.controls.length, 3);
    assert.strictEqual(inMem.controls[2].options.hangingIndent, true, "Hanging indent option should be saved");

    bib = await doc.getBibliography();
    assert.ok(bib);
    assert.strictEqual(bib.internalId, bibId);

    // Test 6: Update bibliography HTML
    await doc.updateBibliographyHtml(bibId, "<p>Turing. (2026). Study of Architecture (2nd ed).</p>", { hangingIndent: false });
    const bibCtrl = inMem.controls.find(c => c.internalId === bibId);
    assert.strictEqual(bibCtrl.html, "<p>Turing. (2026). Study of Architecture (2nd ed).</p>");
    assert.strictEqual(bibCtrl.options.hangingIndent, false, "Updated hanging indent option should be saved");

    // Test 7: Add unrelated third-party control and unlink all
    inMem.controls.push({
        internalId: "other_ctrl_99",
        tag: "SOME_OTHER_PLUGIN_TAG",
        text: "Keep me",
        html: "Keep me"
    });
    assert.strictEqual(inMem.controls.length, 4);

    await doc.unlinkAll();
    assert.strictEqual(inMem.controls.length, 1, "Only non-Mendeley control should remain");
    assert.strictEqual(inMem.controls[0].internalId, "other_ctrl_99", "Unrelated control must not be deleted");

    // Test 8: getDocumentText support
    inMem.docText = "This is a study by (Furlanetto et al., 2016) and (Kuo et al., 2009).";
    const text = await doc.getDocumentText();
    assert.strictEqual(text, "This is a study by (Furlanetto et al., 2016) and (Kuo et al., 2009).");

    console.log("# All tests passed successfully!");
}

runTests().catch(err => {
    console.error("Test failed:", err);
    process.exit(1);
});
