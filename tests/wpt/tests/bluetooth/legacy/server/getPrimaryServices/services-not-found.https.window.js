// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Request for services in a device with no services. Reject ' +
    'with NotFoundError.';
const expected =
    new DOMException('No Services found in device.', 'NotFoundError');

bluetooth_test(async () => {
  let {device} = await getEmptyHealthThermometerDevice();
  return assert_promise_rejects_with_message(
      device.gatt.getPrimaryServices(), expected);
}, test_desc);
