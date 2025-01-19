// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'A device name between 29 and 248 bytes is valid.';
const DEVICE_NAME = 'a_device_name_that_is_longer_than_29_bytes_but_' +
    'shorter_than_248_bytes';

bluetooth_test(async () => {
  let {device} = await setUpPreconnectedFakeDevice({
    fakeDeviceOptions: {name: DEVICE_NAME},
    requestDeviceOptions: {filters: [{name: DEVICE_NAME}]}
  });
  assert_equals(device.name, DEVICE_NAME)
}, test_desc);
