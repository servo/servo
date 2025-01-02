// META: script=/common/get-host-info.sub.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js

const {ORIGIN} = get_host_info();

promise_test_parallel(async t => {
  const iframe = document.createElement("iframe");
  iframe.src = ORIGIN + "/common/blank.html?pipe=status(204)";
  iframe.credentialless = false;
  document.body.appendChild(iframe);
  iframe.credentialless = true;
  iframe.contentWindow.modified = true;
  iframe.src = ORIGIN + "/common/blank.html";
  // Wait for navigation to complete.
  await new Promise(resolve => iframe.onload = resolve);
  assert_true(iframe.credentialless);
  assert_true(iframe.contentWindow.credentialless);
  assert_equals(undefined, iframe.contentWindow.modified);
}, "Credentialless (false => true) => window not reused.");

promise_test_parallel(async t => {
  const iframe = document.createElement("iframe");
  iframe.src = ORIGIN + "/common/blank.html?pipe=status(204)";
  iframe.credentialless = true;
  document.body.appendChild(iframe);
  iframe.credentialless = false;
  iframe.contentWindow.modified = true;
  iframe.src = ORIGIN + "/common/blank.html";
  // Wait for navigation to complete.
  await new Promise(resolve => iframe.onload = resolve);
  assert_false(iframe.credentialless);
  assert_false(iframe.contentWindow.credentialless);
  assert_equals(undefined, iframe.contentWindow.modified);
}, "Credentialless (true => false) => window not reused.");

promise_test_parallel(async t => {
  const iframe = document.createElement("iframe");
  iframe.credentialless = true;
  iframe.src = ORIGIN + "/common/blank.html?pipe=status(204)";
  document.body.appendChild(iframe);
  iframe.credentialless = true;
  iframe.contentWindow.modified = true;
  iframe.src = ORIGIN + "/common/blank.html";
  // Wait for navigation to complete.
  await new Promise(resolve => iframe.onload = resolve);
  assert_true(iframe.credentialless);
  assert_true(iframe.contentWindow.credentialless);
  assert_true(iframe.contentWindow.modified);
}, "Credentialless (true => true) => window reused.");
