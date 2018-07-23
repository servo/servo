// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://wicg.github.io/keyboard-map/

'use strict';

idl_test(
  ['keyboard-map'],
  ['html'],
  async idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
      Keyboard: ['navigator.keyboard'],
      KeyboardLayoutMap: ['layout_map'],
    });

    self.layout_map = await navigator.keyboard.getLayoutMap();
  },
  'Test IDL implementation of Keyboard Map API'
);
