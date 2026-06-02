// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/pointerlock/

idl_test(
  ['pointerlock'],
  ['uievents', 'html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Document: ["window.document"],
      Element: ["window.document.documentElement"],
      MouseEvent: ["new MouseEvent('foo')"]
    });
  }
);
