// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/proximity/

'use strict';

idl_test(
  ['proximity'],
  ['generic-sensor','dom'],
  idl_array => {
    idl_array.add_objects({
      ProximitySensor: ['new ProximitySensor();']
    });
  }
);
