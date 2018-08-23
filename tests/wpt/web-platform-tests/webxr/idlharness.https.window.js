// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://immersive-web.github.io/webxr/

idl_test(
  ['webxr'],
  ['webgl1', 'html', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
      XR: ['navigator.xr'],
    });
  }
);
