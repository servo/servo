// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['storage'],
  ['html'],
  idl_array => {
    idl_array.add_objects({ StorageManager: ['navigator.storage'] });
    if (self.Window) {
      idl_array.add_objects({ Navigator: ['navigator'] });
    } else {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    }
  }
);
