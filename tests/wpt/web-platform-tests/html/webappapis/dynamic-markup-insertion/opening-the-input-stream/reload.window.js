// This test tests for the nonexistence of a reload override buffer, which is
// used in a previous version of the HTML Standard to make reloads of a
// document.open()'d document load the written-to document rather than doing an
// actual reload of the document's URL.
//
// This test has a somewhat interesting structure compared to the other tests
// in this directory. It eschews the <iframe> structure used by other tests,
// since when the child frame is reloaded it would adopt the URL of the test
// page (the responsible document of the entry settings object), and the spec
// forbids navigation in nested browsing contexts to the same URL as their
// parent. To work around that, we use window.open() which does not suffer from
// that restriction.
//
// In any case, this test as the caller of `document.open()` would be used both
// as the test file and as part of the test file. The `if (window.name !==
// "opened-dummy-window")` condition controls what role this file plays.

if (window.name !== "opened-dummy-window") {
  async_test(t => {
    const testURL = document.URL;
    const dummyURL = new URL("resources/dummy.html", document.URL).href;

    // 1. Open an auxiliary window.
    const win = window.open("resources/dummy.html", "opened-dummy-window");
    t.add_cleanup(() => { win.close(); });

    win.addEventListener("load", t.step_func(() => {
      // The timeout seems to be necessary for Firefox, which when `load` is
      // called may still have an active parser.
      t.step_timeout(() => {
        const doc = win.document;
        assert_true(doc.body.textContent.includes("Dummy"), "precondition");
        assert_equals(doc.URL, dummyURL, "precondition");

        window.onChildLoad = t.step_func(message => {
          // 3. The dynamically overwritten content will trigger this function,
          // which puts in place the actual test.

          assert_equals(message, "Written", "script on written page is executed");
          assert_true(win.document.body.textContent.includes("Content"), "page is written to");
          assert_equals(win.document.URL, testURL, "postcondition: after document.write()");
          assert_equals(win.document, doc, "document.open should not change the document object");
          window.onChildLoad = t.step_func_done(message => {
            // 6. This function should be called from the if (opener) branch of
            // this file. It would throw an assertion error if the overwritten
            // content was executed instead.
            assert_equals(message, "Done!", "actual test");
            assert_true(win.document.body.textContent.includes("Back to the test"), "test is reloaded");
            assert_equals(win.document.URL, testURL, "postcondition: after reload");
            assert_not_equals(win.document, doc, "reload should change the document object");
          });

          // 4. Reload the pop-up window. Because of the doc.open() call, this
          // pop-up window will reload to the same URL as this test itself.
          win.location.reload();
        });

        // 2. When it is loaded, dynamically overwrite its content.
        assert_equals(doc.open(), doc);
        assert_equals(doc.URL, testURL, "postcondition: after document.open()");
        doc.write("<p>Content</p><script>opener.onChildLoad('Written');</script>");
        doc.close();
      }, 100);
    }), { once: true });
  }, "Reloading a document.open()'d page should reload the URL of the entry realm's responsible document");
} else {
  document.write("<p>Back to the test</p>");
  // 5. Since this window is window.open()'d, opener refers to the test window.
  // Inform the opener that reload succeeded.
  opener.onChildLoad("Done!");
}
