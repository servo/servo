// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://immersive-web.github.io/webxr-ar-module/

idl_test(
  ['webxr-ar-module'],
  ['webxr', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      // TODO: XRSession
    });
  }
);
