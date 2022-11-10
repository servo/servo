// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Find all services in a device.';

bluetooth_test(async () => {
  let {device} = await getTwoHealthThermometerServicesDevice({
    filters: [{services: ['health_thermometer']}],
    optionalServices: ['generic_access']
  });
  let services = await device.gatt.getPrimaryServices();
  // Expect three service instances.
  assert_equals(services.length, 3);
  services.forEach(s => assert_true(s.isPrimary));

  let uuid_set = new Set(services.map(s => s.uuid));
  // Two of the expected services are 'health_thermometer', so
  // only 2 unique UUIDs.
  assert_equals(uuid_set.size, 2);

  assert_true(uuid_set.has(BluetoothUUID.getService('generic_access')));
  assert_true(uuid_set.has(BluetoothUUID.getService('health_thermometer')));
}, test_desc);
