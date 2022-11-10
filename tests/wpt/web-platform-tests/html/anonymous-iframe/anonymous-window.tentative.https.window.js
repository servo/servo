// META: script=/common/get-host-info.sub.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js

const {ORIGIN} = get_host_info();

promise_test_parallel(async t => {
  const iframe = document.createElement("iframe");
  iframe.src = ORIGIN + "/common/blank.html?pipe=status(204)";
  iframe.anonymous = false;
  document.body.appendChild(iframe);
  iframe.anonymous = true;
  iframe.contentWindow.modified = true;
  iframe.src = ORIGIN + "/common/blank.html";
  // Wait for navigation to complete.
  await new Promise(resolve => iframe.onload = resolve);
  assert_true(iframe.anonymous);
  assert_true(iframe.contentWindow.anonymouslyFramed);
  assert_equals(undefined, iframe.contentWindow.modified);
}, "Anonymous (false => true) => window not reused.");

promise_test_parallel(async t => {
  const iframe = document.createElement("iframe");
  iframe.src = ORIGIN + "/common/blank.html?pipe=status(204)";
  iframe.anonymous = true;
  document.body.appendChild(iframe);
  iframe.anonymous = false;
  iframe.contentWindow.modified = true;
  iframe.src = ORIGIN + "/common/blank.html";
  // Wait for navigation to complete.
  await new Promise(resolve => iframe.onload = resolve);
  assert_false(iframe.anonymous);
  assert_false(iframe.contentWindow.anonymouslyFramed);
  assert_equals(undefined, iframe.contentWindow.modified);
}, "Anonymous (true => false) => window not reused.");

promise_test_parallel(async t => {
  const iframe = document.createElement("iframe");
  iframe.anonymous = true;
  iframe.src = ORIGIN + "/common/blank.html?pipe=status(204)";
  document.body.appendChild(iframe);
  iframe.anonymous = true;
  iframe.contentWindow.modified = true;
  iframe.src = ORIGIN + "/common/blank.html";
  // Wait for navigation to complete.
  await new Promise(resolve => iframe.onload = resolve);
  assert_true(iframe.anonymous);
  assert_true(iframe.contentWindow.anonymouslyFramed);
  assert_true(iframe.contentWindow.modified);
}, "Anonymous (true => true) => window reused.");
