// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Service is removed. Reject with InvalidStateError.';
const expected =
    new DOMException('GATT Service no longer exists.', 'InvalidStateError');

bluetooth_test(async () => {
  const {characteristic, fake_peripheral, fake_service} =
      await getMeasurementIntervalCharacteristic();
  await fake_service.remove();
  await fake_peripheral.simulateGATTServicesChanged();
  await assert_promise_rejects_with_message(
      characteristic.startNotifications(), expected, 'Service got removed.');
}, test_desc);
