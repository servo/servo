// META: script=resources/document-open-side-effects.js

for (const ev of ["unload", "beforeunload", "pagehide"]) {
  async_test(t => {
    const iframe = document.body.appendChild(document.createElement("iframe"));
    t.add_cleanup(() => iframe.remove());
    iframe.src = "/common/blank.html";
    iframe.onload = t.step_func(() => {
      iframe.contentWindow.addEventListener(ev, t.step_func_done(() => {
        const origURL = iframe.contentDocument.URL;
        assertDocumentIsReadyForSideEffectsTest(iframe.contentDocument, `ignore-opens-during-unload counter is greater than 0 during ${ev} event`);
        assert_equals(iframe.contentDocument.open(), iframe.contentDocument);
        assertOpenHasNoSideEffects(iframe.contentDocument, origURL, `ignore-opens-during-unload counter is greater than 0 during ${ev} event`);
      }));
      iframe.src = "about:blank";
    });
  }, `document.open bailout should not have any side effects (ignore-opens-during-unload is greater than 0 during ${ev} event)`);
}
