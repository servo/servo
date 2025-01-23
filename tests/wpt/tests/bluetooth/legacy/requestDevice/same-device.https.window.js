// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Returned device should always be the same.';
let devices = [];

bluetooth_test(async () => {
  await setUpHealthThermometerAndHeartRateDevices();
  devices.push(await requestDeviceWithTrustedClick(
      {filters: [{services: [heart_rate.alias]}]}));
  devices.push(await requestDeviceWithTrustedClick(
      {filters: [{services: [heart_rate.name]}]}));
  devices.push(await requestDeviceWithTrustedClick(
      {filters: [{services: [heart_rate.uuid]}]}));
  assert_equals(devices[0], devices[1]);
  assert_equals(devices[1], devices[2]);
}, test_desc);
