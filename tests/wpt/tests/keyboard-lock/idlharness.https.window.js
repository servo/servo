// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

// https://w3c.github.io/keyboard-lock/

'use strict';

idl_test(
  ['keyboard-lock'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
      Keyboard: ['navigator.keyboard'],
    });
  }
);
