// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['entries-api'],
  ['FileAPI', 'html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      File: ['new File([], "example.txt")'],
      HTMLInputElement: ['document.createElement("input")'],
    });
  }
);
