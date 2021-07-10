// META: script=resources/document-open-side-effects.js

async_test(t => {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  self.testSynchronousScript = t.step_func(() => {
    // Here, the entry settings object is still the iframe's. Delay it in such
    // a way that makes the entry settings object the top-level page's, but
    // without delaying too much that the parser becomes inactive. A microtask
    // is perfect as it's executed in "clean up after running script".
    Promise.resolve().then(t.step_func_done(() => {
      const origURL = iframe.contentDocument.URL;
      assertDocumentIsReadyForSideEffectsTest(iframe.contentDocument, "active parser whose script nesting level is greater than 0");
      assert_equals(iframe.contentDocument.open(), iframe.contentDocument);
      assertOpenHasNoSideEffects(iframe.contentDocument, origURL, "active parser whose script nesting level is greater than 0");
    }));
  });
  iframe.src = "resources/bailout-order-synchronous-script-frame.html";
}, "document.open bailout should not have any side effects (active parser whose script nesting level is greater than 0)");
