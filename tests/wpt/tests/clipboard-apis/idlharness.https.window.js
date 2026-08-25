// META: timeout=long
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['clipboard-apis'],
  ['dom', 'html', 'permissions'],
  idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
      Clipboard: ['navigator.clipboard'],
      ClipboardEvent: ['new ClipboardEvent("x")'],
    });
  }
);
