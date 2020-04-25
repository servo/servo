// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'multiple calls to getDevices() resolves with the same' +
    'BluetoothDevice objects for each granted Bluetooth device.';

bluetooth_test(async () => {
  await getConnectedHealthThermometerDevice();
  let firstDevices = await navigator.bluetooth.getDevices();
  assert_equals(
      firstDevices.length, 1, 'getDevices() should return the granted device.');

  let secondDevices = await navigator.bluetooth.getDevices();
  assert_equals(
      secondDevices.length, 1,
      'getDevices() should return the granted device.');
  assert_equals(
      firstDevices[0], secondDevices[0],
      'getDevices() should produce the same BluetoothDevice objects for a ' +
          'given Bluetooth device.');
}, test_desc);
