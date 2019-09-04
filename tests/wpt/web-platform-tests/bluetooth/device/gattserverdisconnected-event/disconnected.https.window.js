// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = 'A device disconnecting while connected should fire the ' +
    'gattserverdisconnected event.';

bluetooth_test(async () => {
  const {device, fake_peripheral} = await getConnectedHealthThermometerDevice();
  const disconnectPromise = eventPromise(device, 'gattserverdisconnected');

  await fake_peripheral.simulateGATTDisconnection();
  let disconnectEvent = await disconnectPromise;
  assert_true(disconnectEvent.bubbles);
}, test_desc);
