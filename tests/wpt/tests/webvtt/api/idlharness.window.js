// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['webvtt'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      VTTCue: ['new VTTCue(0, 0, "")'],
      VTTRegion: ['new VTTRegion()'],
    });
  }
);
