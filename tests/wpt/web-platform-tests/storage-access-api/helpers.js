'use strict';

function processQueryParams() {
  const queryParams = new URL(window.location).searchParams;
  return {
    expectAccessAllowed: queryParams.get("allowed") != "false",
    topLevelDocument: queryParams.get("rootdocument") != "false",
    testPrefix: queryParams.get("testCase") || "top-level-context",
  };
}

// Create an iframe element, set it up using `setUpFrame`, and optionally fetch
// tests in it. Returns the created frame, after it has loaded.
function CreateFrameHelper(setUpFrame, fetchTests) {
  const frame = document.createElement('iframe');
  const promise = new Promise((resolve, reject) => {
    frame.onload = () => resolve(frame);
    frame.onerror = reject;
  });

  setUpFrame(frame);

  if (fetchTests) {
    fetch_tests_from_window(frame.contentWindow);
  }
  return promise;
}

// Create an iframe element with content loaded from `sourceURL`, append it to
// the document, and optionally fetch tests. Returns the loaded frame, once
// ready.
function CreateFrame(sourceURL, fetchTests = false) {
  return CreateFrameHelper((frame) => {
    frame.src = sourceURL;
    document.body.appendChild(frame);
  }, fetchTests);
}

// Create a new iframe with content loaded from `sourceURL`, and fetches tests.
// Returns the loaded frame, once ready.
function RunTestsInIFrame(sourceURL) {
  return CreateFrame(sourceURL, true);
}

function RunTestsInNestedIFrame(sourceURL) {
  return CreateFrameHelper((frame) => {
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
  }, true);
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

// Sends a message to the given target window, and waits for the provided
// promise to resolve.
function PostMessageAndAwait(message, targetWindow, promise) {
  targetWindow.postMessage(message, "*");
  return promise;
}

// Returns a promise that resolves when the next "reply" is received via
// postMessage.
function ReplyPromise() {
  return new Promise((resolve) => {
    window.addEventListener("message", (event) => {
      resolve(event.data);
    }, { once: true });
  });
}

// Returns a promise that resolves when the given frame fires its load event.
function ReloadPromise(frame) {
  return new Promise((resolve) => {
    frame.addEventListener("load", (event) => {
      resolve();
    }, { once: true });
  });
}

// Reads cookies via document.cookie in the given frame.
function GetJSCookiesFromFrame(frame) {
  return PostMessageAndAwait({ command: "document.cookie" }, frame.contentWindow, ReplyPromise());
}

// Reads cookies via the `httpCookies` variable in the given frame.
function GetHTTPCookiesFromFrame(frame) {
  return PostMessageAndAwait({ command: "httpCookies" }, frame.contentWindow, ReplyPromise());
}

// Executes document.hasStorageAccess in the given frame.
function FrameHasStorageAccess(frame) {
  return PostMessageAndAwait({ command: "hasStorageAccess" }, frame.contentWindow, ReplyPromise());
}

// Executes document.requestStorageAccess in the given frame.
function RequestStorageAccessInFrame(frame) {
  return PostMessageAndAwait({ command: "requestStorageAccess" }, frame.contentWindow, ReplyPromise());
}

// Executes test_driver.set_permission in the given frame, with the provided
// arguments.
function SetPermissionInFrame(frame, args = []) {
  return PostMessageAndAwait({ command: "set_permission", args }, frame.contentWindow, ReplyPromise());
}

// Executes `location.reload()` in the given frame. The returned promise
// resolves when the frame has finished reloading.
function FrameInitiatedReload(frame) {
  return PostMessageAndAwait({ command: "reload" }, frame.contentWindow, ReloadPromise(frame));
}

// Tries to set storage access policy, ignoring any errors.
async function MaybeSetStorageAccess(origin, embedding_origin, value) {
  try {
    await test_driver.set_storage_access(origin, embedding_origin, value);
  } catch (e) {
    // Ignore, can be unimplemented if the platform blocks cross-site cookies
    // by default. If this failed without default blocking we'll notice it later
    // in the test.
  }
}
