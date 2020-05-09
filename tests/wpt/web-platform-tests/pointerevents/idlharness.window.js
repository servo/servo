// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://w3c.github.io/pointerevents/

idl_test(
  ['pointerevents'],
  ['uievents', 'html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Document: ['document'],
      Element: ['document'],
      Window: ['window'],
      Navigator: ['navigator'],
      PointerEvent: ['new PointerEvent("type")']
    });
  }
);
