// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'The Device Information service is composed of blocklisted ' +
    'characteristics so we shouldn\'t find any.';
const expected =
    new DOMException('No Characteristics found in service.', 'NotFoundError');

bluetooth_test(async () => {
  let {device} =
      await getHIDDevice({filters: [{services: ['device_information']}]});
  let service = await device.gatt.getPrimaryService('device_information');
  return assert_promise_rejects_with_message(
      service.getCharacteristics(), expected);
}, test_desc);
