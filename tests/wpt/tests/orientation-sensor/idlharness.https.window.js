// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/orientation-sensor/">

'use strict';

idl_test(
  ['orientation-sensor'],
  ['generic-sensor', 'dom'],
  idl_array => {
    idl_array.add_objects({
      AbsoluteOrientationSensor: ['new AbsoluteOrientationSensor();'],
      RelativeOrientationSensor: ['new RelativeOrientationSensor();']
    });
    idl_array.prevent_multiple_testing('OrientationSensor');
  }
);
