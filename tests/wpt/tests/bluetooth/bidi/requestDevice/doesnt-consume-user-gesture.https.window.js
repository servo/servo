// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'requestDevice calls do not consume user gestures.';

bluetooth_bidi_test(
    () => setUpHealthThermometerAndHeartRateDevices().then(
        () => callWithTrustedClick(async () => {
          for (let i = 0; i < 2; ++i) {
            selectFirstDeviceOnDevicePromptUpdated();
            const device = await navigator.bluetooth.requestDevice(
                {filters: [{services: ['heart_rate']}]});
            assert_equals(device.constructor.name, 'BluetoothDevice');
          }
        })),
    test_desc);
