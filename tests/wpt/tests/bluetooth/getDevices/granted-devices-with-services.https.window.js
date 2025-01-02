// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'getDevices() resolves with permitted devices that can be ' +
    'GATT connected to.';

bluetooth_test(async () => {
  // Set up two connectable Bluetooth devices with their services discovered.
  // One device is a Health Thermometer device with the 'health_thermometer'
  // service while the other is a Heart Rate device with the 'heart_rate'
  // service. Both devices contain the 'generic_access' service.
  let fake_peripherals = await setUpHealthThermometerAndHeartRateDevices();
  for (let fake_peripheral of fake_peripherals) {
    await fake_peripheral.setNextGATTConnectionResponse({code: HCI_SUCCESS});
    await fake_peripheral.addFakeService({uuid: 'generic_access'});
    if (fake_peripheral.address === '09:09:09:09:09:09')
      await fake_peripheral.addFakeService({uuid: 'health_thermometer'});
    else
      await fake_peripheral.addFakeService({uuid: 'heart_rate'});
    await fake_peripheral.setNextGATTDiscoveryResponse({code: HCI_SUCCESS});
  }

  // Request the Health Thermometer device with access to its 'generic_access'
  // service.
  await requestDeviceWithTrustedClick(
      {filters: [{name: 'Health Thermometer', services: ['generic_access']}]});
  let devices = await navigator.bluetooth.getDevices();
  assert_equals(
      devices.length, 1,
      `getDevices() should return the 'Health Thermometer' device.`);

  // Only the 'generic_access' service can be accessed.
  try {
    await devices[0].gatt.connect();
    await devices[0].gatt.getPrimaryService('generic_access');
    assert_promise_rejects_with_message(
        devices[0].gatt.getPrimaryService('health_thermometer'),
        {name: 'SecurityError'});
  } catch (err) {
    assert_unreached(`${err.name}: ${err.message}`);
  }

  // Request the Heart Rate device with access to both of its services.
  await requestDeviceWithTrustedClick({
    filters: [{name: 'Heart Rate', services: ['generic_access', 'heart_rate']}]
  });
  devices = await navigator.bluetooth.getDevices();
  assert_equals(
      devices.length, 2,
      `getDevices() should return the 'Health Thermometer' and 'Health ` +
          `Monitor' devices`);

  // All of Heart Rate device's services can be accessed, while only the
  // 'generic_access' service can be accessed on Health Thermometer.
  try {
    for (let device of devices) {
      await device.gatt.connect();
      await device.gatt.getPrimaryService('generic_access');
      if (device.name === 'Heart Rate') {
        await device.gatt.getPrimaryService('heart_rate');
      } else {
        assert_promise_rejects_with_message(
            devices[0].gatt.getPrimaryService('health_thermometer'),
            {name: 'SecurityError'});
      }
    }
  } catch (err) {
    assert_unreached(`${err.name}: ${err.message}`);
  }
}, test_desc);
