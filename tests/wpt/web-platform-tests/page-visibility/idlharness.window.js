// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/page-visibility/

idl_test(
  ['page-visibility'],
  ['dom', 'html'],
  idl_array => {
    idl_array.add_objects({
      Document: ['document'],
    });
  }
);
