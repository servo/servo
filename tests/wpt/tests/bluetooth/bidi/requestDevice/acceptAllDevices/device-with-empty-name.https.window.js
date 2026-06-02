// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'Device with empty name and no UUIDs nearby. Should be ' +
    'found if acceptAllDevices is true.';

bluetooth_bidi_test(async () => {
  let {device} = await setUpPreconnectedFakeDevice({
    fakeDeviceOptions: {name: ''},
    requestDeviceOptions: {acceptAllDevices: true}
  });
  assert_equals(device.name, '');
}, test_desc);
