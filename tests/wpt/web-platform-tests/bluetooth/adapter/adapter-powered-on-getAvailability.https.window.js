// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'getAvailability() resolves with true if the Bluetooth ' +
    'radio is powered on and the platform supports Bluetooth LE.';

bluetooth_test(async () => {
  await navigator.bluetooth.test.simulateCentral({state: 'powered-on'});
  let availability = await navigator.bluetooth.getAvailability();
  assert_true(
      availability,
      'getAvailability() resolves promise with true when adapter is powered ' +
          'on and it supports Bluetooth Low-Energy.');
}, test_desc);
