// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/pointerevents/

idl_test(
  ['pointerevents'],
  ['uievents', 'dom', 'html'],
  idl_array => {
    idl_array.add_objects({
      Element: ['document'],
      Window: ['window'],
      Navigator: ['navigator'],
      PointerEvent: ['new PointerEvent("type")']
    });
  }
);
