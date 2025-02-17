// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'getAvailability() resolves with false if the system does ' +
    'not have an adapter.';

bluetooth_bidi_test(async () => {
  await test_driver.bidi.bluetooth.simulate_adapter({state: "absent"});
  let availability = await navigator.bluetooth.getAvailability();
  assert_false(
      availability,
      'getAvailability() resolves promise with false when adapter is absent.');
}, test_desc);
