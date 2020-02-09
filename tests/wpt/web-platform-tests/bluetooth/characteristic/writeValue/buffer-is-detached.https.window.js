// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'writeValue() fails when passed a detached buffer';

function detachBuffer(buffer) {
  window.postMessage('', '*', [buffer]);
}

bluetooth_test(async (t) => {
  const {characteristic} = await getMeasurementIntervalCharacteristic();

  const typed_array = Uint8Array.of(1, 2);
  detachBuffer(typed_array.buffer);
  await promise_rejects(
      t, 'InvalidStateError', characteristic.writeValue(typed_array));

  const array_buffer = Uint8Array.of(3, 4).buffer;
  detachBuffer(array_buffer);
  await promise_rejects(
      t, 'InvalidStateError', characteristic.writeValue(array_buffer));
}, test_desc);
