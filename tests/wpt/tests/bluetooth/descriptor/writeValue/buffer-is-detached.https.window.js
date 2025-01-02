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
  const {descriptor, fake_descriptor} = await getUserDescriptionDescriptor();

  let lastValue = await fake_descriptor.getLastWrittenValue();
  assert_equals(lastValue, null);

  await fake_descriptor.setNextWriteResponse(GATT_SUCCESS);

  const typed_array = Uint8Array.of(1, 2);
  detachBuffer(typed_array.buffer);
  await descriptor.writeValue(typed_array);
  lastValue = await fake_descriptor.getLastWrittenValue();
  assert_array_equals(lastValue, []);

  await fake_descriptor.setNextWriteResponse(GATT_SUCCESS);

  const array_buffer = Uint8Array.of(3, 4).buffer;
  detachBuffer(array_buffer);
  await descriptor.writeValue(array_buffer);
  lastValue = await fake_descriptor.getLastWrittenValue();
  assert_array_equals(lastValue, []);
}, test_desc);
