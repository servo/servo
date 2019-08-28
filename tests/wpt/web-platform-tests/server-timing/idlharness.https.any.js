// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/server-timing/

idl_test(
  ['server-timing'],
  ['resource-timing', 'performance-timeline'],
  idl_array => {
    idl_array.add_objects({
      Performance: ['performance'],
    });
  }
);
