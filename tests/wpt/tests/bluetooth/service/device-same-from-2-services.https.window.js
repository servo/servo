// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Same parent device returned from multiple services.';

bluetooth_test(async () => {
  let {device} = await getTwoHealthThermometerServicesDevice(
      {filters: [{services: ['health_thermometer']}]});
  let [service1, service2] =
      await device.gatt.getPrimaryServices('health_thermometer');
  assert_equals(service1.device, service2.device);
}, test_desc);
