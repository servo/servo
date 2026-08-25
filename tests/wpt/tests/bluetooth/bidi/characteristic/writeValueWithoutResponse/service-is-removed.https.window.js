// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'Service gets removed. Reject with InvalidStateError.';
const expected =
    new DOMException('GATT Service no longer exists.', 'InvalidStateError');

bluetooth_bidi_test(async () => {
  const {characteristic, fake_peripheral, fake_service} =
      await getMeasurementIntervalCharacteristic();
  await fake_service.remove();
  await assert_promise_rejects_with_message(
      characteristic.writeValueWithoutResponse(new ArrayBuffer(1 /* length */)),
      expected, 'Service got removed.');
}, test_desc);
