// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/webhid/

idl_test(
  ['webhid'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      HID: ['navigator.hid'],
      Navigator: ['navigator'],
      // TODO: HIDConnectionEvent
      // TODO: HIDInputReportEvent
      // TODO: HIDDevice
    });
  }
);
