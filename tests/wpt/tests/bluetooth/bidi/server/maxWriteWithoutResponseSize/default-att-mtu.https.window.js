// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'maxWriteWithoutResponseSize reflects the default ATT_MTU ' +
    'while no larger MTU has been negotiated.';

bluetooth_bidi_test(async () => {
  const {device} = await getConnectedHealthThermometerDevice();

  // The default ATT_MTU of 23 octets minus the 3 octet ATT Write Command
  // header.
  assert_equals(device.gatt.maxWriteWithoutResponseSize, 20);
}, test_desc);
