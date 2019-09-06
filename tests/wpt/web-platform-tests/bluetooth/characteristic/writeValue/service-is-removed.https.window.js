// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'Service gets removed. Reject with InvalidStateError.';
const expected =
    new DOMException('GATT Service no longer exists.', 'InvalidStateError');

bluetooth_test(async () => {
  const {characteristic, fake_peripheral, fake_service} =
      await getMeasurementIntervalCharacteristic();
  await fake_service.remove();
  await fake_peripheral.simulateGATTServicesChanged();
  await assert_promise_rejects_with_message(
      characteristic.writeValue(new ArrayBuffer(1 /* length */)), expected,
      'Service got removed.');
}, test_desc);
