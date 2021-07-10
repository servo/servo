// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/ua-client-hints/

idl_test(
  ['ua-client-hints'],
  ['html', 'dom'],
  idl_array => {
    if (self.GLOBAL.isWorker()) {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    } else {
      idl_array.add_objects({ Navigator: ['navigator'] });
    }
    idl_array.add_objects({
      NavigatorUAData: ['navigator.userAgentData'],
    });
  }
);
