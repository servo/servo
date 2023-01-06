// META: global=window
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
"use strict";

promise_test(async t => {
  await test_driver.set_permission({ name: "window-placement" }, "denied");

  const status = await navigator.permissions.query({ name:"window-placement" });
  assert_class_string(status, "PermissionStatus");
  assert_equals(status.state, "denied");
}, "Deny window management permission should work.");

promise_test(async t => {
  await test_driver.set_permission({ name: "window-placement" }, "granted");

  const status = await navigator.permissions.query({ name: "window-placement" });
  assert_class_string(status, "PermissionStatus");
  assert_equals(status.state, "granted");
}, "Grant window management permission should work.");
