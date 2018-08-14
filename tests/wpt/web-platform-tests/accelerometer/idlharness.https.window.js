// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/accelerometer/

"use strict";

idl_test(
  ['accelerometer'],
  ['generic-sensor', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Accelerometer: ['new Accelerometer();'],
      LinearAccelerationSensor: ['new LinearAccelerationSensor();'],
      GravitySensor: ['new GravitySensor();']
    });
  }
);
