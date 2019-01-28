// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/web-nfc/

idl_test(
  ['web-nfc'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
      NFC: ['navigator.nfc'],
    });
  }
);
