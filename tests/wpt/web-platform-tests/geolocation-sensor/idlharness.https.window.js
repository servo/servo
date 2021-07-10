// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://wicg.github.io/geolocation-sensor/

'use strict';

idl_test(
  ['geolocation-sensor'],
  ['generic-sensor', 'dom'],
  idl_array => {
    idl_array.add_objects({
      GeolocationSensor: ['new GeolocationSensor'],
    });
  }
);
