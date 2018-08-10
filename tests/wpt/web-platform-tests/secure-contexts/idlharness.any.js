// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webappsec-secure-contexts/

'use strict';

idl_test(
  ['secure-contexts'],
  ['html', 'dom'],
  idl_array => {
    if (self.Window) {
      idl_array.add_objects({ Window: ['self'] });
    } else {
      idl_array.add_objects({ WorkerGlobalScope: ['self'] });
    }
  }
);
