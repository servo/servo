// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Find characteristics with UUID in service.';

bluetooth_test(async () => {
  let {device, fake_peripheral, fake_services} = await getDiscoveredHealthThermometerDevice();
  // Setup a device with two measurement intervals.
  await fake_peripheral.setNextGATTConnectionResponse({code: HCI_SUCCESS});
  await device.gatt.connect();
  let fake_health_thermometer = fake_services.get('health_thermometer');
  await Promise.all([
    fake_health_thermometer.addFakeCharacteristic({
      uuid: 'measurement_interval',
      properties: ['read', 'write', 'indicate']
    }),
    fake_health_thermometer.addFakeCharacteristic({
      uuid: 'measurement_interval',
      properties: ['read', 'write', 'indicate']
    }),
    fake_health_thermometer.addFakeCharacteristic(
        {uuid: 'temperature_measurement', properties: ['indicate']})
  ]);
  await fake_peripheral.setNextGATTDiscoveryResponse({code: HCI_SUCCESS});
  let service = await device.gatt.getPrimaryService('health_thermometer');
  // Actual test starts.
  let characteristics_arrays = await Promise.all([
    service.getCharacteristics(measurement_interval.alias),
    service.getCharacteristics(measurement_interval.name),
    service.getCharacteristics(measurement_interval.uuid)
  ]);
  characteristics_arrays.forEach(characteristics => {
    assert_equals(characteristics.length, 2);
    assert_equals(characteristics[0].uuid, measurement_interval.uuid);
    assert_equals(characteristics[1].uuid, measurement_interval.uuid);
  });
}, test_desc);
