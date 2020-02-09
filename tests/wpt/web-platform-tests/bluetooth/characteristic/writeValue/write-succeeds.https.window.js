// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'A regular write request to a writable characteristic ' +
    'should succeed.';

bluetooth_test(async () => {
  const {characteristic, fake_characteristic} =
      await getMeasurementIntervalCharacteristic();

  let last_value = await fake_characteristic.getLastWrittenValue();
  assert_equals(last_value, null);

  await fake_characteristic.setNextWriteResponse(GATT_SUCCESS);

  const typed_array = Uint8Array.of(1, 2);
  await characteristic.writeValue(typed_array);
  last_value = await fake_characteristic.getLastWrittenValue();
  assert_array_equals(last_value, [1, 2]);

  await fake_characteristic.setNextWriteResponse(GATT_SUCCESS);

  const array_buffer = Uint8Array.of(3, 4).buffer;
  await characteristic.writeValue(array_buffer);
  last_value = await fake_characteristic.getLastWrittenValue();
  assert_array_equals(last_value, [3, 4]);

  await fake_characteristic.setNextWriteResponse(GATT_SUCCESS);

  const data_view = new DataView(new ArrayBuffer(2));
  data_view.setUint8(0, 5);
  data_view.setUint8(1, 6);
  await characteristic.writeValue(data_view);
  last_value = await fake_characteristic.getLastWrittenValue();
  assert_array_equals(last_value, [5, 6]);
}, test_desc);
