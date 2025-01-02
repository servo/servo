// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/ambient-light/

'use strict';

idl_test(
  ['ambient-light'],
  ['generic-sensor', 'dom'],
  idl_array => {
    idl_array.add_objects({
      AmbientLightSensor: ['new AmbientLightSensor()']
    });
  }
);
