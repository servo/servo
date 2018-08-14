// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/pointerevents/extension.html

idl_test(
  ['pointerevents-extension'],
  ['pointerevents', 'uievents', 'dom'],
  idl_array => {
    idl_array.add_objects({
      PointerEvent: ['new PointerEvent("pointer")'],
    })
  }
);
