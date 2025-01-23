// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc =
    'Detached buffers are safe to pass to writeValueWithoutResponse()';

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

  const typed_array = Uint8Array.of(1, 2);
  detachBuffer(typed_array.buffer);
  await characteristic.writeValueWithoutResponse(typed_array);
  ({lastValue, lastWriteType} =
       await fake_characteristic.getLastWrittenValue());
  assert_array_equals(lastValue, []);
  assert_equals(lastWriteType, 'without-response');

  const array_buffer = Uint8Array.of(3, 4).buffer;
  detachBuffer(array_buffer);
  await characteristic.writeValueWithoutResponse(array_buffer);
  ({lastValue, lastWriteType} =
       await fake_characteristic.getLastWrittenValue());
  assert_array_equals(lastValue, []);
  assert_equals(lastWriteType, 'without-response');
}, test_desc);
