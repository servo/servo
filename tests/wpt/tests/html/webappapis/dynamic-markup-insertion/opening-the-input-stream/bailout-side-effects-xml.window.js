// META: script=resources/document-open-side-effects.js

async_test(t => {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  iframe.src = "/common/dummy.xhtml";
  iframe.onload = t.step_func_done(() => {
    const origURL = iframe.contentDocument.URL;
    assertDocumentIsReadyForSideEffectsTest(iframe.contentDocument, "XML document");
    assert_throws_dom(
      "InvalidStateError",
      iframe.contentWindow.DOMException,
      () => {
        iframe.contentDocument.open();
      },
      "document.open() should throw on XML documents"
    );
    assertOpenHasNoSideEffects(iframe.contentDocument, origURL, "XML document");
  });
}, "document.open bailout should not have any side effects (XML document)");
