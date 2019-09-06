// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'Add multiple event listeners then readValue().';

bluetooth_test(async () => {
  const {characteristic, fake_characteristic} =
      await getMeasurementIntervalCharacteristic();
  await fake_characteristic.setNextReadResponse(GATT_SUCCESS, [0, 1, 2]);

  // Make sure that |characteristic.readValue()| resolves after
  // |characteristicvaluechanged| is fired |3| times.
  const results = await assert_promise_resolves_after_event(
      characteristic /* object */, 'readValue' /* func */,
      'characteristicvaluechanged' /* event */, 3 /* num_listeners */);

  const read_value = new Uint8Array(results[0].buffer);
  const event_values = results.slice(1).map(v => new Uint8Array(v.buffer));
  for (const event_value of event_values) {
    assert_equals(event_value.buffer, read_value.buffer);
    assert_array_equals(event_value, read_value);
  }
}, test_desc);
