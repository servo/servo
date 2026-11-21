// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'A reconnected device reports the default ATT_MTU again, ' +
    'because the MTU is negotiated per connection.';

bluetooth_bidi_test(async (t) => {
  const {device, fake_peripheral} = await getConnectedHealthThermometerDevice();
  const watcher =
      new EventWatcher(t, device.gatt, ['maxwritewithoutresponsesizechanged']);

  const sizeChanged = watcher.wait_for('maxwritewithoutresponsesizechanged');
  await fake_peripheral.simulateGATTMtuChange(100);
  await sizeChanged;

  await simulateGATTDisconnectionAndWait(device, fake_peripheral);
  await fake_peripheral.setNextGATTConnectionResponse({code: HCI_SUCCESS});
  await device.gatt.connect();

  assert_equals(device.gatt.maxWriteWithoutResponseSize, 20);
}, test_desc);
