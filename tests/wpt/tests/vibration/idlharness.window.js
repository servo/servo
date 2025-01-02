// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

idl_test(
  ['vibration'],
  ['html'],
  idl_array => {
    idl_array.add_objects({Navigator: ['navigator']});
  }
);
