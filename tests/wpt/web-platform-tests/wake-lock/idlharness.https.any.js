// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/wake-lock/

'use strict';

idl_test(
  ['wake-lock'],
  ['dom', 'html'],
  idl_array => {
    idl_array.add_objects({
      WakeLock: ['new WakeLock("screen")']
    });
  }
);
