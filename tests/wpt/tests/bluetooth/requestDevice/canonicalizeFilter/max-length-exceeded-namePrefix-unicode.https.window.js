// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Unicode string with utf8 representation longer than 248 ' +
    'bytes in \'namePrefix\' must throw NotFoundError.';
const expected = new DOMException(
    'Failed to execute \'requestDevice\' on \'Bluetooth\': ' +
        'A device name can\'t be longer than 248 bytes.',
    new TypeError());
// \u2764's UTF-8 representation is 3 bytes long.
// 83 chars * 3 bytes/char = 249 bytes
const unicode_name = '\u2764'.repeat(83);

bluetooth_test(
    () => assert_promise_rejects_with_message(
        requestDeviceWithTrustedClick({filters: [{namePrefix: unicode_name}]}),
        expected),
    test_desc);
