// META: title=Language Model From Detached Iframe
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  assert_true(!!LanguageModel);
  // Create the iframe and append it to the document.
  let iframe = document.createElement("iframe");
  document.childNodes[document.childNodes.length - 1].appendChild(iframe);
  let iframeWindow = iframe.contentWindow;
  iframeWindow.languageModel = iframeWindow.LanguageModel;
  let iframeDOMException = iframeWindow.DOMException;
  // Detach the iframe.
  iframe.remove();
  // Calling `LanguageModel.availability()` from an invalid script state will trigger
  // the "The execution context is not valid." exception.
  await promise_rejects_dom(
    t, 'InvalidStateError', iframeDOMException, iframeWindow.languageModel.availability(),
    "The promise should be rejected with InvalidStateError if the execution context is invalid."
  );
});
