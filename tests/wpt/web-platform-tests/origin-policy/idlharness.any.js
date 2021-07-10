// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

idl_test(
  ['origin-policy'],
  ['html', 'dom'],
  idl_array => {
    if (self.Window) {
      idl_array.add_objects({ Window: ['self'] });
    } else {
      idl_array.add_objects({ WorkerGlobalScope: ['self'] });
    }
  }
);
