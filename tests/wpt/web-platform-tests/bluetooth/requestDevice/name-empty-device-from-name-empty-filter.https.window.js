// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'An empty name device can be obtained by empty name filter.'

bluetooth_test(async () => {
  let {device} = await setUpPreconnectedFakeDevice({
    fakeDeviceOptions: {name: ''},
    requestDeviceOptions: {filters: [{name: ''}]}
  });
  assert_equals(device.name, '');
}, test_desc);
