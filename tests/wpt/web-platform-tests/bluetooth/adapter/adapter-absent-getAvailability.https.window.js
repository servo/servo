// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'getAvailability() resolves with false if the system does ' +
    'not have an adapter.';

bluetooth_test(async () => {
  await navigator.bluetooth.test.simulateCentral({state: 'absent'});
  let availability = await navigator.bluetooth.getAvailability();
  assert_false(
      availability,
      'getAvailability() resolves promise with false when adapter is absent.');
}, test_desc);
