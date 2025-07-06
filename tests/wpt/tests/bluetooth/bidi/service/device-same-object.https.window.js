// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = '[SameObject] test for BluetoothRemoteGATTService device.';

bluetooth_bidi_test(async () => {
  let {device} = await getHealthThermometerDevice(
      {filters: [{services: ['health_thermometer']}]});
  let service = await device.gatt.getPrimaryService('health_thermometer');
  assert_equals(service.device, device);
}, test_desc);
