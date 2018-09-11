// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['InputDeviceCapabilities'],
  ['uievents', 'dom'],
  idl_array => {
    idl_array.add_objects({
      InputDeviceCapabilities: ["new InputDeviceCapabilities"],
    });
  }
);
