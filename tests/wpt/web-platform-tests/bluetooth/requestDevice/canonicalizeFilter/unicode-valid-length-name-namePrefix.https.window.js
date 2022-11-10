// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'A namePrefix containing unicode characters whose utf8 ' +
    'length is less than 30 must not throw an error.';
// \u2764's UTF-8 representation is 3 bytes long.
// 9 chars * 3 bytes/char = 27 bytes
const valid_unicode_name = '\u2764'.repeat(9);

bluetooth_test(async () => {
  let {device} = await setUpPreconnectedFakeDevice({
    fakeDeviceOptions: {name: valid_unicode_name},
    requestDeviceOptions: {filters: [{namePrefix: valid_unicode_name}]}
  });
  device => assert_equals(device.name, valid_unicode_name)
}, test_desc);
