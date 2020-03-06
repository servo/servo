// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// See: https://wicg.github.io/shape-detection-api/

'use strict';

idl_test(
  ['shape-detection-api'],
  ['dom', 'geometry'],
  async idl_array => {
    idl_array.add_objects({
      FaceDetector: ['new FaceDetector()'],
      BarcodeDetector: ['new BarcodeDetector()'],
    });
  }
);
