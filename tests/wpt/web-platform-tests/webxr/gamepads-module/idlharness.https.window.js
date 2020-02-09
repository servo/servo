// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://immersive-web.github.io/webxr-gamepads-module/

idl_test(
  ['webxr-gamepads-module'],
  ['webxr', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      // TODO: XRInputSource
    });
  }
);
