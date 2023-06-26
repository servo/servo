// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Device will connect';

bluetooth_test(async () => {
  let {device, fake_peripheral} = await getDiscoveredHealthThermometerDevice();
  await fake_peripheral.setNextGATTConnectionResponse({code: HCI_SUCCESS});
  let gatt = await device.gatt.connect();
  assert_true(gatt.connected);
}, test_desc);
