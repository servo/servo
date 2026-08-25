// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/gyroscope/

'use strict';

idl_test(
  ['gyroscope'],
  ['generic-sensor', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Gyroscope: ['new Gyroscope();']
    });
  }
);
