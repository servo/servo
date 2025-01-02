// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Reject with NotFoundError if Bluetooth is not supported.';
const expected =
    new DOMException('Bluetooth Low Energy not available.', 'NotFoundError');

bluetooth_test(
    () => navigator.bluetooth.test.setLESupported(false).then(
        () => assert_promise_rejects_with_message(
            requestDeviceWithTrustedClick({acceptAllDevices: true}), expected,
            'Bluetooth Low Energy is not supported.')),
    test_desc);
