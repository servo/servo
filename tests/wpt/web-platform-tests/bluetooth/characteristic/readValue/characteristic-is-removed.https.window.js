// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'Characteristic gets removed. Reject with InvalidStateError.';
const expected = new DOMException(
    'GATT Characteristic no longer exists.', 'InvalidStateError');

bluetooth_test(async () => {
  const {characteristic, fake_characteristic} =
      await getMeasurementIntervalCharacteristic();
  await fake_characteristic.remove();
  await assert_promise_rejects_with_message(
      characteristic.readValue(), expected, 'Characteristic got removed.');
}, test_desc);
