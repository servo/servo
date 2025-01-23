// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'An empty |filters| member should result in a TypeError';
const expected = new DOMException(
    'Failed to execute \'requestDevice\' on ' +
        '\'Bluetooth\': \'filters\' member must be non-empty to ' +
        'find any devices.',
    new TypeError());

bluetooth_test(
    () => assert_promise_rejects_with_message(
        requestDeviceWithTrustedClick({filters: []}), expected),
    test_desc);
