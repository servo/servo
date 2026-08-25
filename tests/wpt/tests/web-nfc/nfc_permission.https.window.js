// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

promise_test(async t => {
  await test_driver.set_permission({name: 'nfc'}, 'denied');

  const status = await navigator.permissions.query({name: 'nfc'});
  assert_class_string(status, 'PermissionStatus');
  assert_equals(status.state, 'denied');
}, 'Deny nfc permission should work.');

promise_test(async t => {
  await test_driver.set_permission({name: 'nfc'}, 'granted');

  const status = await navigator.permissions.query({name: 'nfc'});
  assert_class_string(status, 'PermissionStatus');
  assert_equals(status.state, 'granted');
}, 'Grant nfc permission should work.');
