// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['webxrlayers'],
  ['webxr', 'webgl1', 'webgl2', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      // TODO: Add object instances
    });
  },
  'WebXR Layers Module IDL Test'
);
