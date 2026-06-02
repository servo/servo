// META: timeout=long
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['virtual-keyboard.tentative'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
      VirtualKeyboard: ['navigator.virtualKeyboard'],
      GeometryChangeEvent: ['new GeometryChangeEvent("x")'],
    });
  }
);
