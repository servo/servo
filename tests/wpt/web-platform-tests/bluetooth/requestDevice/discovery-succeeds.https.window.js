// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Discover a device using alias, name, or UUID.';

const test_specs = [
  {
    filters: [{services: [health_thermometer.alias]}],
  },
  {
    filters: [{services: [health_thermometer.name]}],
  },
  {
    filters: [{services: [health_thermometer.uuid]}],
  },
];

bluetooth_test(
    () => setUpHealthThermometerDevice().then(() => {
      let test_promises = Promise.resolve();
      test_specs.forEach(args => {
        test_promises = test_promises.then(async () => {
          const device = await requestDeviceWithTrustedClick(args);
          assert_equals(device.constructor.name, 'BluetoothDevice');
        });
      });
      return test_promises;
    }),
    test_desc);
