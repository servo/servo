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

// Sends a message to the given target window and returns a promise that
// resolves when a reply was sent.
function PostMessageAndAwaitReply(message, targetWindow) {
  const timestamp = window.performance.now();
  const reply = ReplyPromise(timestamp);
  targetWindow.postMessage({timestamp, ...message}, "*");
  return reply;
}

// Returns a promise that resolves when the next "reply" is received via
// postMessage. Takes a "timestamp" argument to validate that the received
// message belongs to its original counterpart.
function ReplyPromise(timestamp) {
  return new Promise((resolve) => {
    const listener = (event) => {
      if (event.data.timestamp == timestamp) {
        window.removeEventListener("message", listener);
        resolve(event.data.data);
      }
    };
    window.addEventListener("message", listener);
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
  return PostMessageAndAwaitReply(
      { command: "document.cookie" }, frame.contentWindow);
}

// Reads cookies via the `httpCookies` variable in the given frame.
function GetHTTPCookiesFromFrame(frame) {
  return PostMessageAndAwaitReply(
      { command: "httpCookies" }, frame.contentWindow);
}

// Executes document.hasStorageAccess in the given frame.
function FrameHasStorageAccess(frame) {
  return PostMessageAndAwaitReply(
      { command: "hasStorageAccess" }, frame.contentWindow);
}

// Executes document.requestStorageAccess in the given frame.
function RequestStorageAccessInFrame(frame) {
  return PostMessageAndAwaitReply(
      { command: "requestStorageAccess" }, frame.contentWindow);
}

// Executes test_driver.set_permission in the given frame, with the provided
// arguments.
function SetPermissionInFrame(frame, args = []) {
  return PostMessageAndAwaitReply(
      { command: "set_permission", args }, frame.contentWindow);
}

// Waits for a storage-access permission change and resolves with the current
// state.
function ObservePermissionChange(frame, args = []) {
  return PostMessageAndAwaitReply(
      { command: "observe_permission_change", args }, frame.contentWindow);
}

// Executes `location.reload()` in the given frame. The returned promise
// resolves when the frame has finished reloading.
function FrameInitiatedReload(frame) {
  const reload = ReloadPromise(frame);
  frame.contentWindow.postMessage({ command: "reload" }, "*");
  return reload;
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
