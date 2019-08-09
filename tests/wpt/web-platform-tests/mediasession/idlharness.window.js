// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/mediasession/

'use strict';

idl_test(
  ['mediasession'],
  ['html'],
  idl_array => {
    idl_array.add_objects({
      MediaMetadata: ['new MediaMetadata()'],
      MediaSession: ['navigator.mediaSession'],
      Navigator: ['navigator']
    });
  }
);
