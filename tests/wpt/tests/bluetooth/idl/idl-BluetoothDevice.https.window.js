// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc_idl = 'BluetoothDevice IDL test.';

test(() => {
  assert_throws_js(
      TypeError, () => new BluetoothDevice(),
      'the constructor should not be callable with "new"');
  assert_throws_js(
      TypeError, () => BluetoothDevice(),
      'the constructor should not be callable');
}, test_desc_idl);

const test_desc_attr = 'BluetoothDevice attributes.';
let device;

bluetooth_test(async () => {
  let {device} = await getConnectedHealthThermometerDevice();

  assert_equals(device.constructor.name, 'BluetoothDevice');
  var old_device_id = device.id;
  assert_throws_js(
      TypeError, () => device.id = 'overwritten',
      'the device id should not be writable');
  assert_throws_js(
      TypeError, () => device.name = 'overwritten',
      'the device name should not be writable');
  assert_throws_js(
      TypeError, () => device.watchingAdvertisements = true,
      'the device watchingAdvertisements should not be writable');
  assert_equals(device.id, old_device_id);
  assert_equals(device.name, 'Health Thermometer');
  assert_equals(device.watchingAdvertisements, false);
}, test_desc_attr);
