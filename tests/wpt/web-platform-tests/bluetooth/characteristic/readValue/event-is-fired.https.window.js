// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'Reading a characteristic should fire an event.';

bluetooth_test(async () => {
  const {characteristic, fake_characteristic} =
      await getMeasurementIntervalCharacteristic();
  await fake_characteristic.setNextReadResponse(GATT_SUCCESS, [0, 1, 2]);

  // Make sure that |characteristic.readValue()| resolves after
  // |characteristicvaluechanged| is fired.
  const results = await assert_promise_resolves_after_event(
      characteristic /* object */, 'readValue' /* func */,
      'characteristicvaluechanged' /* event */);

  const read_value = new Uint8Array(results[0].buffer);
  const event_value = new Uint8Array(results[1].buffer);
  assert_equals(event_value.buffer, read_value.buffer);
  assert_array_equals(event_value, read_value);
}, test_desc);
