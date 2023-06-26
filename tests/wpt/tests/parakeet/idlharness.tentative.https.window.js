// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
'use strict';

idl_test(
  ['parakeet.tentative'],
  ['html'],
  idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
    });
  }
);
