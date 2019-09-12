// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'getAvailability() resolves with false after the powered ' +
    'on adapter is removed.';

bluetooth_test(async () => {
  const fake_central =
      await navigator.bluetooth.test.simulateCentral({state: 'powered-on'});
  let availability = await navigator.bluetooth.getAvailability();
  assert_true(
      availability,
      'getAvailability() resolves promise with true when adapter is powered ' +
          'on and it supports Bluetooth Low-Energy.');

  await fake_central.setState('absent');
  availability = await navigator.bluetooth.getAvailability();
  assert_false(
      availability,
      'getAvailability() resolves promise with false after adapter has been ' +
          'has been removed.');
}, test_desc);
