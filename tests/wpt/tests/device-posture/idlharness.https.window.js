// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/device-posture/

'use strict';

idl_test(["device-posture"], ["html", "dom", "webidl"], (idl_array) => {
  idl_array.add_objects({
    DevicePosture: ["navigator.devicePosture"],
  });
});
