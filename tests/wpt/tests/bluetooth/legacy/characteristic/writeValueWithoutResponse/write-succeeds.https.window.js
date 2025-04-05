// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'A regular write request to a writable characteristic ' +
    'should succeed.';

bluetooth_test(async () => {
  const {characteristic, fake_characteristic} =
      await getMeasurementIntervalCharacteristic();

  let lastValue, lastWriteType;
  ({lastValue, lastWriteType} =
       await fake_characteristic.getLastWrittenValue());
  assert_equals(lastValue, null);
  assert_equals(lastWriteType, 'none');

  const typed_array = Uint8Array.of(1, 2);
  await characteristic.writeValueWithoutResponse(typed_array);
  ({lastValue, lastWriteType} =
       await fake_characteristic.getLastWrittenValue());
  assert_array_equals(lastValue, [1, 2]);
  assert_equals(lastWriteType, 'without-response');

  const array_buffer = Uint8Array.of(3, 4).buffer;
  await characteristic.writeValueWithoutResponse(array_buffer);
  ({lastValue, lastWriteType} =
       await fake_characteristic.getLastWrittenValue());
  assert_array_equals(lastValue, [3, 4]);
  assert_equals(lastWriteType, 'without-response');

  const data_view = new DataView(new ArrayBuffer(2));
  data_view.setUint8(0, 5);
  data_view.setUint8(1, 6);
  await characteristic.writeValueWithoutResponse(data_view);
  ({lastValue, lastWriteType} =
       await fake_characteristic.getLastWrittenValue());
  assert_array_equals(lastValue, [5, 6]);
  assert_equals(lastWriteType, 'without-response');
}, test_desc);
