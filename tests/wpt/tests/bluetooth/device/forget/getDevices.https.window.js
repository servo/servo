// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js

bluetooth_test(async () => {
  await getConnectedHealthThermometerDevice();
  const devicesBeforeForget = await navigator.bluetooth.getDevices();
  assert_equals(
    devicesBeforeForget.length, 1, 'getDevices() should return the granted device.');

  const device = devicesBeforeForget[0];
  await device.forget();
  const devicesAfterForget = await navigator.bluetooth.getDevices();
  assert_equals(
    devicesAfterForget.length, 0,
      'getDevices() is empty after device.forget().');

  // Call forget() again getDevices() should return the same result of empty
  // list.
  await device.forget();
  const devicesAfterForgetCalledAgain = await navigator.bluetooth.getDevices();
  assert_equals(
      devicesAfterForgetCalledAgain.length, 0,
      'getDevices() is still empty after device.forget().');
}, 'forget() removes devices from getDevices().');
