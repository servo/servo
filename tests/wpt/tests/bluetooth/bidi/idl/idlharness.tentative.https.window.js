// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://bluetooth.spec.whatwg.org/scanning.html

idl_test(['bluetooth'], ['dom', 'html', 'permissions'], idl_array => {
  try {
    self.event = new BluetoothAdvertisingEvent('type');
  } catch (e) {
    // Surfaced when 'event' is undefined below.
  }

  idl_array.add_objects({
    Navigator: ['navigator'],
    Bluetooth: ['navigator.bluetooth'],
    BluetoothAdvertisingEvent: ['event'],
  });
});
