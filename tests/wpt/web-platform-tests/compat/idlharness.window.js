// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://compat.spec.whatwg.org/

idl_test(
  ['compat'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Window: ['window'],
      HTMLBodyElement: ['document.body'],
    });
  }
);
