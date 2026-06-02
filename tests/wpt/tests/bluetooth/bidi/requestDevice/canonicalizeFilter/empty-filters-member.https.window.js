// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'An empty |filters| member should result in a TypeError';
const expected = new DOMException(
    'Failed to execute \'requestDevice\' on ' +
        '\'Bluetooth\': \'filters\' member must be non-empty to ' +
        'find any devices.',
    new TypeError());

bluetooth_bidi_test(
    () => assert_promise_rejects_with_message(
        requestDeviceWithTrustedClick({filters: []}), expected),
    test_desc);
