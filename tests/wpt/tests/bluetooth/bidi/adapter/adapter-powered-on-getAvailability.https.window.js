// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'getAvailability() resolves with true if the Bluetooth ' +
    'radio is powered on and the platform supports Bluetooth LE.';

bluetooth_bidi_test(async () => {
  await test_driver.bidi.bluetooth.simulate_adapter({state: "powered-on"});
  let availability = await navigator.bluetooth.getAvailability();
  assert_true(
      availability,
      'getAvailability() resolves promise with true when adapter is powered ' +
          'on and it supports Bluetooth Low-Energy.');
}, test_desc);
