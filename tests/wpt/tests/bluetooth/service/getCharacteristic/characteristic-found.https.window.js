// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Request for characteristic. Should return right ' +
    'characteristic.';

bluetooth_test(async () => {
  let {device} = await getHealthThermometerDevice();
  let service = await device.gatt.getPrimaryService('health_thermometer');
  let characteristics = await Promise.all([
    service.getCharacteristic(measurement_interval.alias),
    service.getCharacteristic(measurement_interval.name),
    service.getCharacteristic(measurement_interval.uuid)
  ]);
  characteristics.forEach(characteristic => {
    assert_equals(
        characteristic.uuid, measurement_interval.uuid,
        'Characteristic UUID should be the same as requested UUID.');
    assert_equals(
        characteristic.service, service,
        'Characteristic service should be the same as service.');
  });
}, test_desc);
