'use strict';

function processQueryParams() {
  const queryParams = new URL(window.location).searchParams;
  return {
    expectAccessAllowed: queryParams.get("allowed") != "false",
    topLevelDocument: queryParams.get("rootdocument") != "false",
    testPrefix: queryParams.get("testCase") || "top-level-context",
  };
}

function CreateFrameAndRunTests(setUpFrame) {
  const frame = document.createElement('iframe');
  const promise = new Promise((resolve, reject) => {
    frame.onload = resolve;
    frame.onerror = reject;
  });

  setUpFrame(frame);

  fetch_tests_from_window(frame.contentWindow);
  return promise;
}

function RunTestsInIFrame(sourceURL) {
  return CreateFrameAndRunTests((frame) => {
    frame.src = sourceURL;
    document.body.appendChild(frame);
  });
}

function RunTestsInNestedIFrame(sourceURL) {
  return CreateFrameAndRunTests((frame) => {
    document.body.appendChild(frame);
    frame.contentDocument.write(`
      <script src="/resources/testharness.js"></script>
      <script src="helpers.js"></script>
      <body>
      <script>
        RunTestsInIFrame("${sourceURL}");
      </script>
    `);
    frame.contentDocument.close();
  });
}

function RunRequestStorageAccessInDetachedFrame() {
  const frame = document.createElement('iframe');
  document.body.append(frame);
  const inner_doc = frame.contentDocument;
  frame.remove();
  return inner_doc.requestStorageAccess();
}

function RunRequestStorageAccessViaDomParser() {
  const parser = new DOMParser();
  const doc = parser.parseFromString('<html></html>', 'text/html');
  return doc.requestStorageAccess();
}

function RunCallbackWithGesture(callback) {
  return test_driver.bless('run callback with user gesture', callback);
}
