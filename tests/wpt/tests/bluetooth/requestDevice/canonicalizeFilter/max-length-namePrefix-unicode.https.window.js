// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'A unicode device namePrefix of 248 bytes is valid.';
// \u00A1's UTF-8 representation is 2 bytes long.
// 124 chars * 2 bytes/char = 248 bytes
const DEVICE_NAME = '\u00A1'.repeat(124);

bluetooth_test(async () => {
  let {device} = await setUpPreconnectedFakeDevice({
    fakeDeviceOptions: {name: DEVICE_NAME},
    requestDeviceOptions: {filters: [{namePrefix: DEVICE_NAME}]}
  });
  device => assert_equals(device.name, DEVICE_NAME)
}, test_desc);
