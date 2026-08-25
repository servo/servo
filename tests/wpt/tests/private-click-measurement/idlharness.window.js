// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['private-click-measurement'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      HTMLAnchorElement: ['document.createElement("a")'],
    });
  }
);
