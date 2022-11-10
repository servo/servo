// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Find correct services with UUID.';
let device, fake_peripheral;

bluetooth_test(async () => {
  let {device, fake_peripheral} = await getConnectedHealthThermometerDevice(
      {filters: [{services: ['health_thermometer']}]});
  let fake_service =
      await fake_peripheral.addFakeService({uuid: 'health_thermometer'});
  await Promise.all([
    fake_service.addFakeCharacteristic(
        {uuid: 'temperature_measurement', properties: ['indicate']}),
    fake_service.addFakeCharacteristic(
        {uuid: 'temperature_measurement', properties: ['indicate']})
  ]);
  await fake_peripheral.setNextGATTDiscoveryResponse({code: HCI_SUCCESS});
  let services = await device.gatt.getPrimaryServices('health_thermometer');
  let [characteristics1, characteristics2] = await Promise.all(
      [services[0].getCharacteristics(), services[1].getCharacteristics()]);
  if (characteristics1.length === 2)
    assert_equals(characteristics2.length, 3);
  else if (characteristics2.length === 2)
    assert_equals(characteristics1.length, 3);
  else
    assert_unreached('Invalid lengths.');
}, test_desc);
