// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'If a site disconnects from a device while the platform is ' +
    'disconnecting that device, only one gattserverdisconnected event should ' +
    'fire.';

bluetooth_test(async () => {
  const {device, fake_peripheral} = await getConnectedHealthThermometerDevice();
  let num_events = 0;

  // 1. Listen for disconnections.
  device.addEventListener('gattserverdisconnected', () => num_events++);

  // 2. Disconnect several times.
  await Promise.all([
    eventPromise(device, 'gattserverdisconnected'),
    fake_peripheral.simulateGATTDisconnection(),
    device.gatt.disconnect(),
    device.gatt.disconnect(),
  ]);

  // 3. Wait to catch disconnect events.
  await new Promise(resolve => step_timeout(resolve, 50));

  // 4. Ensure there is exactly 1 disconnection recorded.
  assert_equals(num_events, 1);
}, test_desc);
