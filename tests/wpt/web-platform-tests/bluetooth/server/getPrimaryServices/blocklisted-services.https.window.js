// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Request for services. Does not return blocklisted service.';

bluetooth_test(async () => {
  let {device} = await getHIDDevice({
    filters: [{services: ['device_information']}],
    optionalServices: ['generic_access', 'human_interface_device']
  })
  let services = await device.gatt.getPrimaryServices();
  assert_equals(services.length, 2);
  let uuid_set = new Set(services.map(s => s.uuid));

  assert_equals(uuid_set.size, 2);
  assert_true(uuid_set.has(BluetoothUUID.getService('generic_access')));
  assert_true(uuid_set.has(BluetoothUUID.getService('device_information')));
  assert_false(
      uuid_set.has(BluetoothUUID.getService('human_interface_device')));
}, test_desc);
