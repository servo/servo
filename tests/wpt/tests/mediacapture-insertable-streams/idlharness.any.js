// META: global=dedicatedworker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

idl_test(
  ['mediacapture-transform'],
  ['dom', 'html'],
  idl_array => {
    idl_array.add_objects({
      VideoTrackGenerator: ['new VideoTrackGenerator()'],
    });
  }
);
