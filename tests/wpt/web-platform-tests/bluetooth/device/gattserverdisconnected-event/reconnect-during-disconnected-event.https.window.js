// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'A device that reconnects during the ' +
    'gattserverdisconnected event should still receive ' +
    'gattserverdisconnected events after re-connection.';

bluetooth_test(async () => {
  const {device, fake_peripheral} = await getConnectedHealthThermometerDevice();

  const reconnectPromise = new Promise(async (resolve) => {
    device.addEventListener('gattserverdisconnected', async () => {
      // 2. Reconnect.
      await fake_peripheral.setNextGATTConnectionResponse({
        code: HCI_SUCCESS,
      });
      await device.gatt.connect();

      // 3. Disconnect after reconnecting.
      const disconnectPromise = eventPromise(device, 'gattserverdisconnected');
      fake_peripheral.simulateGATTDisconnection();
      resolve(disconnectPromise);
    }, {once: true});
  });

  // 1. Disconnect.
  await fake_peripheral.simulateGATTDisconnection();
  await reconnectPromise;
}, test_desc);
