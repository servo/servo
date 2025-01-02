// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/magnetometer/

'use strict';

idl_test(
  ['magnetometer'],
  ['generic-sensor', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Magnetometer: ['new Magnetometer();'],
      UncalibratedMagnetometer: ['new UncalibratedMagnetometer();']
    });
  }
);
