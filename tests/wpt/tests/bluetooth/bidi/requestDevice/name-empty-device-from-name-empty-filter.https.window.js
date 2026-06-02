// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'An empty name device can be obtained by empty name filter.'

bluetooth_bidi_test(async () => {
  let {device} = await setUpPreconnectedFakeDevice({
    fakeDeviceOptions: {name: ''},
    requestDeviceOptions: {filters: [{name: ''}]}
  });
  assert_equals(device.name, '');
}, test_desc);
