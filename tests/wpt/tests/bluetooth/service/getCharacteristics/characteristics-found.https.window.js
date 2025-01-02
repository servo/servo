// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Find all characteristics in a service.';

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
  let characteristics = await service.getCharacteristics();
  // Expect three characteristic instances.
  assert_equals(characteristics.length, 3);

  let uuid_set = new Set(characteristics.map(c => c.uuid));
  // Two of the expected characteristics are
  // 'measurement_interval', so only 2 unique UUID.
  assert_equals(uuid_set.size, 2);
  assert_true(
      uuid_set.has(BluetoothUUID.getCharacteristic('measurement_interval')));
  assert_true(
      uuid_set.has(BluetoothUUID.getCharacteristic('temperature_measurement')));
}, test_desc);
