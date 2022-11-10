// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Request for services. Should return right number of ' +
    'services.';

bluetooth_test(async () => {
  let {device} = await getTwoHealthThermometerServicesDevice(
      {filters: [{services: ['health_thermometer']}]});
  let services_arrays = await Promise.all([
    device.gatt.getPrimaryServices(health_thermometer.alias),
    device.gatt.getPrimaryServices(health_thermometer.name),
    device.gatt.getPrimaryServices(health_thermometer.uuid)
  ]);
  services_arrays.forEach(services => {
    assert_equals(services.length, 2);
    services.forEach(service => {
      assert_equals(
          service.uuid, BluetoothUUID.getService('health_thermometer'));
      assert_true(service.isPrimary);
    });
  })
}, test_desc);
