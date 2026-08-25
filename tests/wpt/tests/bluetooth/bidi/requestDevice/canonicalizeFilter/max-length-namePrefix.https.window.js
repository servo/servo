// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'A device namePrefix of 248 bytes is valid.';
const DEVICE_NAME = 'a'.repeat(248);

bluetooth_bidi_test(async () => {
  let {device} = await setUpPreconnectedFakeDevice({
    fakeDeviceOptions: {name: DEVICE_NAME},
    requestDeviceOptions: {filters: [{namePrefix: DEVICE_NAME}]}
  });
  device => assert_equals(device.name, DEVICE_NAME)
}, test_desc);
