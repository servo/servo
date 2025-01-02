// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/screen-orientation/

idl_test(
  ['screen-orientation'],
  ['dom', 'cssom-view', 'html'],
  idl_array => {
    idl_array.add_objects({
      Screen: ['screen'],
      ScreenOrientation: ['screen.orientation']
    });
  }
);
