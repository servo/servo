// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc =
    'An empty |exclusionFilters| member should result in a TypeError';
const expected = new DOMException(
    'Failed to execute \'requestDevice\' on ' +
        '\'Bluetooth\': \'exclusionFilters\' member must be non-empty to ' +
        'exclude any device.',
    new TypeError());

bluetooth_bidi_test(
    () => assert_promise_rejects_with_message(
        requestDeviceWithTrustedClick(
            {filters: [{name: 'Name'}], exclusionFilters: []}),
        expected),
    test_desc);
