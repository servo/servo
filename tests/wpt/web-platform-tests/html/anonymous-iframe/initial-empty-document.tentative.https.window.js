// META: script=/common/get-host-info.sub.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js

const {ORIGIN} = get_host_info();

promise_test_parallel(async t => {
  const parent = document.createElement("iframe");
  parent.anonymous = true;
  document.body.appendChild(parent);
  parent.src = ORIGIN + "/common/blank.html";
  // Wait for navigation to complete.
  await new Promise(resolve => parent.onload = resolve);
  assert_true(parent.anonymous);

  const child = document.createElement("iframe");
  parent.contentDocument.body.appendChild(child);
  assert_false(child.anonymous);
  assert_true(child.contentWindow.anonymouslyFramed);
}, "Initial empty document inherits from parent's document.");

promise_test_parallel(async t => {
  const parent = document.createElement("iframe");
  document.body.appendChild(parent);
  parent.src = ORIGIN + "/common/blank.html";
  // Wait for navigation to complete.
  await new Promise(resolve => parent.onload = resolve);
  assert_false(parent.anonymous);

  const child = document.createElement("iframe");
  child.anonymous = true;
  parent.contentDocument.body.appendChild(child);
  assert_true(child.anonymous);
  assert_true(child.contentWindow.anonymouslyFramed);
}, "Initial empty document inherits from its's iframe's anonymous attribute.");
