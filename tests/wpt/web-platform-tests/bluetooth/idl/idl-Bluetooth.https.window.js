// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';
const test_desc = 'Bluetooth IDL test';

test(() => {
  assert_throws_js(
      TypeError, () => new Bluetooth(),
      'the constructor should not be callable with "new"');
  assert_throws_js(
      TypeError, () => Bluetooth(),
      'the constructor should not be callable');

  // Bluetooth implements BluetoothDiscovery;
  assert_true('requestDevice' in navigator.bluetooth);
  assert_true('getDevices' in navigator.bluetooth);
  assert_equals(navigator.bluetooth.requestDevice.length, 0);
}, test_desc);