// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/audio-session/

'use strict';

idl_test(
  ['audio-session'],
  ['dom', 'html'],
  idl_array => {
    idl_array.add_objects({
      AudioSession: ['navigator.audioSession'],
      Navigator: ['navigator'],
    });
  }
);
