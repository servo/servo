// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Multiple connects should return the same gatt object.';

bluetooth_test(async () => {
  let {device, fake_peripheral} = await getDiscoveredHealthThermometerDevice();
  await fake_peripheral.setNextGATTConnectionResponse({code: HCI_SUCCESS});
  // No second response is necessary because an ATT Bearer
  // already exists from the first connection.
  // See
  // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
  // step 5.1.
  let gatt1 = await device.gatt.connect();
  let gatt2 = await device.gatt.connect();
  assert_equals(gatt1, gatt2);
}, test_desc);
