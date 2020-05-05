// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://webbluetoothcg.github.io/web-bluetooth/

idl_test(
  ['web-bluetooth'],
  ['dom', 'html', 'permissions'],
  idl_array => {
    try {
      self.event = new BluetoothAdvertisingEvent('type');
    } catch(e) {
      // Surfaced when 'event' is undefined below.
    }

    idl_array.add_objects({
      Navigator: ['navigator'],
      Bluetooth: ['navigator.bluetooth'],
      BluetoothAdvertisingEvent: ['event'],
    });
  }
);
