// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'A device name prefix longer than 248 must reject.';
const expected = new DOMException(
    'Failed to execute \'requestDevice\' on \'Bluetooth\': A device ' +
        'name can\'t be longer than 248 bytes.',
    new TypeError());
const name_too_long = 'a'.repeat(249);

bluetooth_test(
    () => assert_promise_rejects_with_message(
        requestDeviceWithTrustedClick({filters: [{namePrefix: name_too_long}]}),
        expected, 'Device name longer than 248'),
    test_desc);
