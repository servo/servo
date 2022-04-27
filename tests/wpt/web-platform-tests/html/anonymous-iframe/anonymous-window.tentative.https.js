// META: script=/common/get-host-info.sub.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js

const ORIGIN = get_host_info();

promise_test_parallel(async t => {
  const iframe = document.createElement("iframe");
  document.body.appendChild(iframe);
  iframe.contentWindow.modified = true;
  iframe.anonymous = true;
  iframe.src = ORIGIN + "/blank.html";
  // Wait for navigation finished.
  await t.step_wait(() =>
    iframe.contentWindow.location.href === iframe.src,
    "Wait for the navigation to complete");
  assert_true(iframe.anonymous);
  assert_equals(undefined, iframe.contentWindow.modified);
}, "Anonymous (false => true) => window not reused.");

promise_test_parallel(async t => {
  const iframe = document.createElement("iframe");
  iframe.anonymous = true;
  document.body.appendChild(iframe);
  iframe.contentWindow.modified = true;
  iframe.anonymous = false;
  iframe.src = ORIGIN + "/blank.html";
  // Wait for navigation finished.
  await t.step_wait(() =>
    iframe.contentWindow.location.href === iframe.src,
    "Wait for the navigation to complete");
  assert_false(iframe.anonymous);
  assert_equals(undefined, iframe.contentWindow.modified);
}, "Anonymous (true => false) => window not reused.");

promise_test_parallel(async t => {
  const iframe = document.createElement("iframe");
  iframe.anonymous = true;
  document.body.appendChild(iframe);
  iframe.contentWindow.modified = true;
  iframe.src = ORIGIN + "/blank.html";
  // Wait for navigation finished.
  await t.step_wait(() =>
    iframe.contentWindow.location.href === iframe.src,
    "Wait for the navigation to complete");
  assert_true(iframe.anonymous);
  assert_true(iframe.contentWindow.modified);
}, "Anonymous (true => true) => window reused.");
