// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'A read request succeeds and returns the characteristic\'s ' +
    'value.';
const EXPECTED_VALUE = [0, 1, 2];

bluetooth_test(async () => {
  const {characteristic, fake_characteristic} =
      await getMeasurementIntervalCharacteristic();
  await fake_characteristic.setNextReadResponse(GATT_SUCCESS, EXPECTED_VALUE);
  const value = await characteristic.readValue();
  assert_array_equals(new Uint8Array(value.buffer), EXPECTED_VALUE)
}, test_desc);
