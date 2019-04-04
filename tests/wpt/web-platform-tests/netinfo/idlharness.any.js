// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/netinfo/

idl_test(
  ['netinfo'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({ NetworkInformation: ['navigator.connection'] });
    if (self.GLOBAL.isWorker()) {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    } else {
      idl_array.add_objects({ Navigator: ['navigator'] });
    }
  }
);
