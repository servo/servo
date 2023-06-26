// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
const test_desc = 'Succesful read should update characteristic.value';
const EXPECTED_VALUE = [0, 1, 2];

bluetooth_test(async () => {
  const {characteristic, fake_characteristic} =
      await getMeasurementIntervalCharacteristic();
  assert_equals(characteristic.value, null);

  await fake_characteristic.setNextReadResponse(GATT_SUCCESS, EXPECTED_VALUE);
  await characteristic.readValue();
  assert_array_equals(
      new Uint8Array(characteristic.value.buffer), EXPECTED_VALUE)
}, test_desc);
