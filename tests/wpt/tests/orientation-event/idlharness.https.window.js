// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

// https://w3c.github.io/deviceorientation/spec-source-orientation.html

'use strict';

idl_test(
  ['orientation-event'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Window: ['window'],
      DeviceOrientationEvent: ['new DeviceOrientationEvent("foo")'],
      DeviceMotionEvent: ['new DeviceMotionEvent("foo")'],
    });
  }
);
