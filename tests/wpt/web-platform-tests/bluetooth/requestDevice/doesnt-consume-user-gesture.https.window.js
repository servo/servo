// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'requestDevice calls do not consume user gestures.';

bluetooth_test(
    () => setUpHealthThermometerAndHeartRateDevices().then(
        () => callWithTrustedClick(() => {
          let first = navigator.bluetooth.requestDevice(
              {filters: [{services: ['heart_rate']}]});
          let second = navigator.bluetooth.requestDevice(
              {filters: [{services: ['heart_rate']}]});
          return Promise.all([
            first.then(
                device =>
                    assert_equals(device.constructor.name, 'BluetoothDevice')),
            second.then(
                device =>
                    assert_equals(device.constructor.name, 'BluetoothDevice')),
          ]);
        })),
    test_desc);
