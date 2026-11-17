// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'A negotiated ATT_MTU updates maxWriteWithoutResponseSize ' +
    'and fires maxwritewithoutresponsesizechanged.';

bluetooth_bidi_test(async (t) => {
  const {device, fake_peripheral} = await getConnectedHealthThermometerDevice();
  const watcher =
      new EventWatcher(t, device.gatt, ['maxwritewithoutresponsesizechanged']);

  const sizeChanged = watcher.wait_for('maxwritewithoutresponsesizechanged');
  await fake_peripheral.simulateGATTMtuChange(100);
  await sizeChanged;

  assert_equals(device.gatt.maxWriteWithoutResponseSize, 97);
}, test_desc);
