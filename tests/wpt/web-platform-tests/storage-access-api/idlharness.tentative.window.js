// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
'use strict';

idl_test(
  ['storage-access-api.tentative'],
  ['dom'],
  idl_array => {
    idl_array.add_objects({
      Document: ['document'],
    });
  }
);
