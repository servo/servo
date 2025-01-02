// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

idl_test(
  ['permissions-request'],
  ['permissions', 'html', 'dom'],
  async idl_array => {
    if (self.GLOBAL.isWorker()) {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    } else {
      idl_array.add_objects({ Navigator: ['navigator'] });
    }

    idl_array.add_objects({
      Permissions: ['navigator.permissions'],
    });
  }
);
