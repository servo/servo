// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Reject with NotFoundError if there is no BT radio present.';
const expected =
    new DOMException('Bluetooth adapter not available.', 'NotFoundError');

bluetooth_test(
    () => navigator.bluetooth.test.simulateCentral({state: 'absent'})
              .then(
                  () => assert_promise_rejects_with_message(
                      requestDeviceWithTrustedClick(
                          {filters: [{services: ['generic_access']}]}),
                      expected, 'Bluetooth adapter is not present.')),
    test_desc);
