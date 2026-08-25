// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['wai-aria'],
  ['dom'],
  idl_array => {
    idl_array.add_objects({
      Element: ['element'],
    });
    self.element = document.createElementNS(null, "test");
  }
);
