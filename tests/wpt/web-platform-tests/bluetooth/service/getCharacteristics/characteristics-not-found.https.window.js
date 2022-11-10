// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Request for absent characteristics. Reject with ' +
    'NotFoundError.';
const expected =
    new DOMException('No Characteristics found in service.', 'NotFoundError');

bluetooth_test(async () => {
  let {service} = await getEmptyHealthThermometerService();
  return assert_promise_rejects_with_message(
      service.getCharacteristics(), expected);
}, test_desc);
