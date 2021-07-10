// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

idl_test(
  ['input-device-capabilities'],
  ['uievents', 'dom'],
  idl_array => {
    idl_array.add_objects({
      InputDeviceCapabilities: ["new InputDeviceCapabilities"],
    });
  }
);
