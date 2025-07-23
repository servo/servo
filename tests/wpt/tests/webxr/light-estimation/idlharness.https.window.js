// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['webxr-lighting-estimation'],
  ['webxr', 'webxrlayers', 'webgl1', 'webgl2', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      // TODO: Add object instances
    });
  },
  'WebXR Lighting Estimation Module IDL Test'
);
