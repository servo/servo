// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Detached buffers are safe to pass to writeValue()';

function detachBuffer(buffer) {
  window.postMessage('', '*', [buffer]);
}

bluetooth_test(async (t) => {
  const {characteristic, fake_characteristic} =
      await getMeasurementIntervalCharacteristic();

  let lastValue, lastWriteType;
  ({lastValue, lastWriteType} =
       await fake_characteristic.getLastWrittenValue());
  assert_equals(lastValue, null);
  assert_equals(lastWriteType, 'none');

  await fake_characteristic.setNextWriteResponse(GATT_SUCCESS);

  const typed_array = Uint8Array.of(1, 2);
  detachBuffer(typed_array.buffer);
  await characteristic.writeValue(typed_array);
  ({lastValue, lastWriteType} =
       await fake_characteristic.getLastWrittenValue());
  assert_array_equals(lastValue, []);
  assert_equals(lastWriteType, 'default-deprecated');

  await fake_characteristic.setNextWriteResponse(GATT_SUCCESS);

  const array_buffer = Uint8Array.of(3, 4).buffer;
  detachBuffer(array_buffer);
  await characteristic.writeValue(array_buffer);
  ({lastValue, lastWriteType} =
       await fake_characteristic.getLastWrittenValue());
  assert_array_equals(lastValue, []);
  assert_equals(lastWriteType, 'default-deprecated');
}, test_desc);
