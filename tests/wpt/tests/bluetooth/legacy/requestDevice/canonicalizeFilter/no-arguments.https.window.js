// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'requestDevice() requires an argument.';
const expected = new TypeError();

promise_test(
    () => assert_promise_rejects_with_message(
        requestDeviceWithTrustedClick(), expected),
    test_desc);
