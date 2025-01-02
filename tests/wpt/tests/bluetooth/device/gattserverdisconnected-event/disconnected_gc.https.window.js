// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/gc.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'A device disconnecting after the BluetoothDevice object ' +
    'has been GC\'ed should not access freed memory.';

bluetooth_test(async () => {
  let {fake_peripheral} = await getConnectedHealthThermometerDevice();

  // 1. Disconnect.
  await fake_peripheral.simulateGATTDisconnection();

  // 2. Run garbage collection.
  fake_peripheral = undefined;
  await garbageCollect();

  // 3. Wait 50ms after the GC runs for the disconnection event to come back.
  // There's nothing to assert other than that only valid memory is used.
  await new Promise(resolve => step_timeout(resolve, 50));
}, test_desc);
