// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'getAvailability() resolves with true after adapter is ' +
    'inserted into a system with a platform that supports Bluetooth LE.';

bluetooth_test(async () => {
  const fake_central =
      await navigator.bluetooth.test.simulateCentral({state: 'absent'});
  let availability = await navigator.bluetooth.getAvailability();
  assert_false(
      availability,
      'getAvailability() resolves promise with false when adapter is absent.');

  await fake_central.setState('powered-on');
  availability = await navigator.bluetooth.getAvailability();
  assert_true(
      availability,
      'getAvailability() resolves promise with true after Bluetooth LE ' +
          'capable adapter has been has been added.');
}, test_desc);
