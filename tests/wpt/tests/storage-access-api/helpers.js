'use strict';

function processQueryParams() {
  const url = new URL(window.location);
  const queryParams = url.searchParams;
  return {
    topLevelDocument: window === window.top,
    testPrefix: queryParams.get("testCase") || "top-level-context",
  };
}

// Create an iframe element, set it up using `setUpFrame`, and optionally fetch
// tests in it. Returns the created frame, after it has loaded.
async function CreateFrameHelper(setUpFrame, fetchTests) {
  const frame = document.createElement('iframe');
  const promise = new Promise((resolve, reject) => {
    frame.onload = () => resolve(frame);
    frame.onerror = reject;
  });

  setUpFrame(frame);

  if (fetchTests) {
    await fetch_tests_from_window(frame.contentWindow);
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

function CreateDetachedFrame() {
  const frame = document.createElement('iframe');
  document.body.append(frame);
  const inner_doc = frame.contentDocument;
  frame.remove();
  return inner_doc;
}

function CreateDocumentViaDOMParser() {
  const parser = new DOMParser();
  const doc = parser.parseFromString('<html></html>', 'text/html');
  return doc;
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
function LoadPromise(frame) {
  return new Promise((resolve) => {
    frame.addEventListener("load", (event) => {
      resolve();
    }, { once: true });
  });
}

// Writes cookies via document.cookie in the given frame.
function SetDocumentCookieFromFrame(frame, cookie) {
  return PostMessageAndAwaitReply(
    { command: "write document.cookie", cookie }, frame.contentWindow);
}

// Reads cookies via document.cookie in the given frame.
function GetJSCookiesFromFrame(frame) {
  return PostMessageAndAwaitReply(
      { command: "document.cookie" }, frame.contentWindow);
}

async function DeleteCookieInFrame(frame, name, params) {
  await SetDocumentCookieFromFrame(frame, `${name}=0; expires=${new Date(0).toUTCString()}; ${params};`);
  assert_false(cookieStringHasCookie(name, '0', await GetJSCookiesFromFrame(frame)), `Verify that cookie '${name}' has been deleted.`);
}

// Tests whether the frame can write cookies via document.cookie. Note that this
// overwrites, then optionally deletes, cookies named "cookie" and "foo".
//
// This function requires the caller to have included
// /cookies/resources/cookie-helper.sub.js.
async function CanFrameWriteCookies(frame, keep_after_writing = false) {
  const cookie_suffix = "Secure;SameSite=None;Path=/";
  await DeleteCookieInFrame(frame, "cookie", cookie_suffix);
  await DeleteCookieInFrame(frame, "foo", cookie_suffix);

  await SetDocumentCookieFromFrame(frame, `cookie=monster;${cookie_suffix}`);
  await SetDocumentCookieFromFrame(frame, `foo=bar;${cookie_suffix}`);

  const cookies = await GetJSCookiesFromFrame(frame);
  const can_write = cookieStringHasCookie("cookie", "monster", cookies) &&
      cookieStringHasCookie("foo", "bar", cookies);

  if (!keep_after_writing) {
    await DeleteCookieInFrame(frame, "cookie", cookie_suffix);
    await DeleteCookieInFrame(frame, "foo", cookie_suffix);
  }

  return can_write;
}

// Tests whether the current frame can read and write cookies via HTTP headers.
// This deletes, writes, reads, then deletes a cookie named "cookie".
async function CanAccessCookiesViaHTTP() {
  await create_cookie(window.location.origin, "cookie", "1", "samesite=None;Secure");
  const http_cookies = await fetch(`${window.location.origin}/storage-access-api/resources/echo-cookie-header.py`)
      .then((resp) => resp.text());
  const can_access = cookieStringHasCookie("cookie", "1", http_cookies);

  erase_cookie_from_js("cookie", "SameSite=None;Secure;Path=/");

  return can_access;
}

// Tests whether the current frame can read and write cookies via
// document.cookie. This deletes, writes, reads, then deletes a cookie named
// "cookie".
function CanAccessCookiesViaJS() {
  erase_cookie_from_js("cookie", "SameSite=None;Secure;Path=/");
  assert_false(cookieStringHasCookie("cookie", "1", document.cookie));

  document.cookie = "cookie=1;SameSite=None;Secure;Path=/";
  const can_access = cookieStringHasCookie("cookie", "1", document.cookie);

  erase_cookie_from_js("cookie", "SameSite=None;Secure;Path=/");
  assert_false(cookieStringHasCookie("cookie", "1", document.cookie));

  return can_access;
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
  const reload = LoadPromise(frame);
  frame.contentWindow.postMessage({ command: "reload" }, "*");
  return reload;
}

// Executes `location.href = url` in the given frame. The returned promise
// resolves when the frame has finished navigating.
function FrameInitiatedNavigation(frame, url) {
  const load = LoadPromise(frame);
  frame.contentWindow.postMessage({ command: "navigate", url }, "*");
  return load;
}

// Makes a subresource request to the provided host in the given frame, and
// returns the cookies that were included in the request.
function FetchSubresourceCookiesFromFrame(frame, host) {
  return FetchFromFrame(frame, `${host}/storage-access-api/resources/echo-cookie-header.py`);
}

// Makes a subresource request to the provided host in the given frame, and
// returns the response.
function FetchFromFrame(frame, url) {
  return PostMessageAndAwaitReply(
    { command: "cors fetch", url }, frame.contentWindow);
}

// Tries to set storage access policy, ignoring any errors.
//
// Note: to discourage the writing of tests that assume unpartitioned cookie
// access by default, any test that calls this with `value` == "blocked" should
// do so as the first step in the test.
async function MaybeSetStorageAccess(origin, embedding_origin, value) {
  try {
    await test_driver.set_storage_access(origin, embedding_origin, value);
  } catch (e) {
    // Ignore, can be unimplemented if the platform blocks cross-site cookies
    // by default. If this failed without default blocking we'll notice it later
    // in the test.
  }
}
