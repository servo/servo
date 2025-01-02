// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/webhid/

idl_test(
  ['webhid'],
  ['html', 'dom'],
  idl_array => {
    if (self.GLOBAL.isWindow()) {
      idl_array.add_objects({ Navigator: ['navigator'] });
    } else if (self.GLOBAL.isWorker()) {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    }

    idl_array.add_objects({
      HID: ['navigator.hid'],
      // TODO: HIDConnectionEvent
      // TODO: HIDInputReportEvent
      // TODO: HIDDevice
    });
  }
);
